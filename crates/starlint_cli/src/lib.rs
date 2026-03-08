//! Library root for starlint CLI.
//!
//! Orchestrates CLI parsing, config loading, session construction, and linting.

pub mod cli;
pub mod error;

use std::fmt::Write;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;

use cli::{Cli, Command, OutputFormatArg};
use starlint_config::resolve::load_config;
use starlint_core::diagnostic::OutputFormat;
use starlint_core::engine::{FileDiagnostics, LintSession};
use starlint_core::file_discovery::discover_files;
use starlint_core::fix::apply_fixes;
use starlint_loader::all_rule_metas;
use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::FixKind;

/// Result of running the linter, used to determine process exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// No errors found.
    Success,
    /// Lint errors found.
    LintErrors,
}

/// Diagnostic counts after linting.
struct DiagnosticCounts {
    /// Total error-severity diagnostics.
    errors: usize,
    /// Total warning-severity diagnostics.
    warnings: usize,
}

/// Run the starlint CLI.
///
/// Parses arguments, loads config, discovers files, lints, and formats output.
/// Returns the exit status (caller decides the exit code).
#[allow(clippy::too_many_lines, clippy::print_stderr)]
pub fn run() -> miette::Result<ExitStatus> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let args = Cli::parse();

    let output_format = match args.format {
        OutputFormatArg::Pretty => OutputFormat::Pretty,
        OutputFormatArg::Json => OutputFormat::Json,
        OutputFormatArg::Compact => OutputFormat::Compact,
        OutputFormatArg::Count => OutputFormat::Count,
    };

    // Determine fix mode from command or flags.
    let (paths, fix_enabled, fix_dangerous) = match &args.command {
        Some(Command::Fix {
            paths, dangerous, ..
        }) => (paths.clone(), true, *dangerous),
        Some(Command::Lint { paths }) => (paths.clone(), args.fix, args.fix_dangerous),
        Some(Command::Lsp) => return run_lsp(),
        Some(Command::Init) => {
            run_init()?;
            return Ok(ExitStatus::Success);
        }
        Some(Command::Rules { plugin, json }) => {
            run_rules(plugin.as_deref(), *json);
            return Ok(ExitStatus::Success);
        }
        None => (args.paths.clone(), args.fix, args.fix_dangerous),
    };

    let config = load_merged_config(args.config.as_deref())?;
    configure_thread_pool(args.threads, config.settings.threads);

    let total_start = Instant::now();

    // Discover files.
    let discover_start = Instant::now();
    let files = discover_files(&paths);
    let discover_elapsed = discover_start.elapsed();

    if files.is_empty() {
        eprintln!("warning: no lintable files found");
        return Ok(ExitStatus::Success);
    }

    tracing::debug!("found {} files to lint", files.len());

    // Load all plugins (native + WASM) through unified loader.
    let setup_start = Instant::now();
    let loaded = starlint_loader::load_plugins(&config);
    tracing::debug!("loaded {} plugin(s)", loaded.plugins.len());
    let override_set = starlint_core::overrides::OverrideSet::compile(&config.overrides);
    let session = LintSession::new(loaded.plugins, output_format)
        .with_severity_overrides(loaded.severity_overrides)
        .with_override_set(override_set)
        .with_disabled_rules(loaded.disabled_rules);
    let setup_elapsed = setup_start.elapsed();

    // Lint files.
    let lint_start = Instant::now();
    let results = session.lint_files(&files);
    let lint_elapsed = lint_start.elapsed();

    if fix_enabled {
        apply_fixes_to_files(&results, fix_dangerous, &session);
    }

    let report_start = Instant::now();
    let counts = report_diagnostics(&results, output_format);
    let report_elapsed = report_start.elapsed();

    if args.timing {
        print_timing_detailed(
            &total_start,
            &discover_elapsed,
            &setup_elapsed,
            &lint_elapsed,
            &report_elapsed,
            files.len(),
        );
    }

    // Enforce max-warnings threshold.
    if args.max_warnings > 0 && counts.warnings > args.max_warnings {
        eprintln!(
            "{} warning(s) exceed max-warnings threshold ({})",
            counts.warnings, args.max_warnings,
        );
        return Ok(ExitStatus::LintErrors);
    }

    if counts.errors > 0 {
        Ok(ExitStatus::LintErrors)
    } else {
        Ok(ExitStatus::Success)
    }
}

/// Load config from explicit path or auto-discover, falling back to defaults.
///
/// Returns an error if a config file exists but fails to parse.
/// Returns `Config::default()` only when no config file is found.
fn load_merged_config(
    explicit_path: Option<&Path>,
) -> Result<starlint_config::Config, error::CliError> {
    if let Some(path) = explicit_path {
        return Ok(load_config(path)?);
    }

    match starlint_config::resolve::find_config_file(Path::new(".")) {
        Some(path) => Ok(load_config(&path)?),
        None => Ok(starlint_config::Config::default()),
    }
}

/// Configure the rayon global thread pool. CLI arg takes priority over config.
fn configure_thread_pool(cli_threads: usize, config_threads: usize) {
    let thread_count = if cli_threads > 0 {
        cli_threads
    } else {
        config_threads
    };
    if thread_count > 0 {
        if let Err(err) = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
        {
            tracing::warn!("failed to set thread count to {thread_count}: {err}");
        }
    }
}

/// Maximum number of lint-fix passes per file before giving up.
///
/// Prevents infinite loops when fixes oscillate. In practice, two passes
/// are enough: the first applies non-overlapping fixes, the second picks up
/// any that were skipped due to span overlaps.
const MAX_FIX_PASSES: usize = 10;

/// Apply fixes from diagnostics and write fixed files back to disk.
///
/// Uses a multi-pass convergence loop: after applying fixes, the file is
/// re-linted and any remaining fixable diagnostics are applied again. This
/// handles cases where two rules produce overlapping fixes (e.g.,
/// `no-console` removes a statement while `no-console-spaces` fixes the
/// inner string) — the skipped fix is picked up on the next pass.
///
/// When `include_dangerous` is false, only `SafeFix` fixes are applied.
/// When true, all fixes (including `SuggestionFix` and `DangerousFix`) are applied.
#[allow(clippy::print_stderr)]
fn apply_fixes_to_files(
    results: &[FileDiagnostics],
    include_dangerous: bool,
    session: &LintSession,
) {
    let mut files_fixed = 0usize;

    for result in results {
        let mut source = result.source_text.clone();
        let mut changed = false;

        for pass in 0..MAX_FIX_PASSES {
            let diagnostics = if pass == 0 {
                result.diagnostics.clone()
            } else {
                session.lint_single_file(&result.path, &source).diagnostics
            };

            let filtered = filter_fixable_diags(&diagnostics, include_dangerous);
            if filtered.is_empty() {
                break;
            }

            let fixed = apply_fixes(&source, &filtered);
            if fixed == source {
                break;
            }
            source = fixed;
            changed = true;
        }

        if !changed {
            continue;
        }

        let dir = result.path.parent().unwrap_or_else(|| Path::new("."));
        match write_atomic(dir, &result.path, &source) {
            Ok(()) => {
                files_fixed = files_fixed.saturating_add(1);
            }
            Err(err) => {
                eprintln!(
                    "warning: failed to write fix for {}: {err}",
                    result.path.display()
                );
            }
        }
    }

    if files_fixed > 0 {
        eprintln!("fixed {files_fixed} file(s)");
    }
}

/// Filter diagnostics to only those with applicable fixes.
///
/// Reads `fix.kind` directly from each diagnostic's fix — no metadata lookup needed.
fn filter_fixable_diags(
    diagnostics: &[starlint_plugin_sdk::diagnostic::Diagnostic],
    include_dangerous: bool,
) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.fix.as_ref().is_some_and(|f| match f.kind {
                FixKind::SafeFix => true,
                FixKind::SuggestionFix | FixKind::DangerousFix => include_dangerous,
                FixKind::None => false,
            })
        })
        .cloned()
        .collect()
}

/// Count errors/warnings and write diagnostics to stdout.
///
/// Counting is parallelized across files. Formatting streams directly to
/// stdout per-file (no intermediate `String` buffer). For `Count` mode,
/// formatting is skipped entirely — only the summary line is printed.
#[allow(clippy::print_stdout, clippy::print_stderr)]
fn report_diagnostics(
    results: &[FileDiagnostics],
    output_format: OutputFormat,
) -> DiagnosticCounts {
    use rayon::prelude::*;

    // Phase 1: Count severities in parallel (cheap — no formatting).
    let (total_errors, total_warnings) = results
        .par_iter()
        .map(|result| {
            let mut errors = 0usize;
            let mut warnings = 0usize;
            for diag in &result.diagnostics {
                match diag.severity {
                    Severity::Error => {
                        errors = errors.saturating_add(1);
                    }
                    Severity::Warning => {
                        warnings = warnings.saturating_add(1);
                    }
                    Severity::Suggestion => {}
                }
            }
            (errors, warnings)
        })
        .reduce(
            || (0, 0),
            |(e1, w1), (e2, w2)| (e1.saturating_add(e2), w1.saturating_add(w2)),
        );

    // Phase 2: Format and write sequentially (skip entirely for Count mode).
    if output_format != OutputFormat::Count {
        let stdout = std::io::stdout();
        let mut writer = std::io::BufWriter::new(stdout.lock());
        for result in results {
            #[allow(clippy::let_underscore_must_use)]
            let _ = starlint_core::diagnostic::write_diagnostics(
                &mut writer,
                &result.diagnostics,
                &result.source_text,
                &result.path,
                output_format,
            );
        }
        #[allow(clippy::let_underscore_must_use)]
        let _ = writer.flush();
    }

    if total_errors > 0 || total_warnings > 0 {
        eprintln!(
            "found {total_errors} error(s) and {total_warnings} warning(s) in {} file(s)",
            results.len()
        );
    }

    DiagnosticCounts {
        errors: total_errors,
        warnings: total_warnings,
    }
}

/// Print detailed timing information to stderr.
#[allow(clippy::print_stderr)] // Timing is metadata, goes to stderr
fn print_timing_detailed(
    total_start: &Instant,
    discover_elapsed: &std::time::Duration,
    setup_elapsed: &std::time::Duration,
    lint_elapsed: &std::time::Duration,
    report_elapsed: &std::time::Duration,
    file_count: usize,
) {
    let total_elapsed = total_start.elapsed();
    let total_secs = total_elapsed.as_secs_f64();
    #[allow(clippy::cast_precision_loss)]
    let files_per_sec = if total_secs > 0.0 {
        f64::from(u32::try_from(file_count).unwrap_or(u32::MAX)) / total_secs
    } else {
        0.0
    };
    eprintln!(
        "timing: discovery {:.1}ms, plugins {:.1}ms, lint {:.1}ms, report {:.1}ms, total {:.1}ms ({:.0} files/s)",
        discover_elapsed.as_secs_f64() * 1000.0,
        setup_elapsed.as_secs_f64() * 1000.0,
        lint_elapsed.as_secs_f64() * 1000.0,
        report_elapsed.as_secs_f64() * 1000.0,
        total_secs * 1000.0,
        files_per_sec,
    );
}

/// Atomic counter for unique temp file names within a process.
static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Write content to a file atomically via a temp file and rename.
///
/// Uses PID + atomic counter to avoid collisions between threads or
/// concurrent starlint processes.
fn write_atomic(dir: &Path, target: &Path, content: &str) -> std::io::Result<()> {
    let seq = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp_path = dir.join(format!(".starlint-fix-{}-{seq}.tmp", std::process::id()));
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, target)?;
    Ok(())
}

/// Initialize a default `starlint.toml` config file.
#[allow(clippy::print_stdout, clippy::print_stderr)]
fn run_init() -> Result<(), error::CliError> {
    let config_path = PathBuf::from("starlint.toml");
    if config_path.exists() {
        eprintln!("warning: starlint.toml already exists");
        return Ok(());
    }

    let default_config = r#"# starlint configuration
# See https://github.com/ogghead/starlint for documentation

[settings]
threads = 0  # 0 = auto-detect

# Plugins provide lint rules. All built-in plugins are enabled by default
# when this section is omitted. List plugins explicitly to control which
# are active:
# [plugins]
# core = true          # General JS/TS rules
# react = true         # React + JSX A11y + React Perf
# typescript = true    # TypeScript rules
# testing = true       # Jest + Vitest
# modules = true       # Import + Node + Promise
# nextjs = true        # Next.js rules
# vue = true           # Vue rules
# jsdoc = true         # JSDoc rules
# storybook = true     # Storybook rules
# custom = { path = "./plugins/my-plugin.wasm" }  # External WASM

# Per-rule severity overrides.
# Note: Adding any rule here disables all other built-in rules not listed.
# To keep all defaults, leave the [rules] section empty.
[rules]
# "no-debugger" = "error"
"#;

    std::fs::write(&config_path, default_config)
        .map_err(|err| error::CliError::Init(err.to_string()))?;

    println!("created starlint.toml");
    Ok(())
}

/// List available rules, optionally filtered by plugin name.
#[allow(clippy::print_stdout)]
fn run_rules(plugin_filter: Option<&str>, json: bool) {
    let metas = all_rule_metas();

    // Filter by plugin prefix if specified (e.g. "storybook" matches "storybook/*" rules).
    let filtered: Vec<_> = if let Some(plugin) = plugin_filter {
        let prefix = format!("{plugin}/");
        metas
            .into_iter()
            .filter(|m| m.name.starts_with(&prefix))
            .collect()
    } else {
        metas
    };

    if json {
        #[allow(clippy::let_underscore_must_use)]
        if let Ok(json_str) = serde_json::to_string_pretty(&filtered) {
            println!("{json_str}");
        }
    } else {
        let mut output = String::new();
        for meta in &filtered {
            #[allow(clippy::let_underscore_must_use)]
            let _ = writeln!(
                output,
                "  {:<30} {:<15} {}",
                meta.name,
                category_label(&meta.category),
                meta.description
            );
        }
        if !output.is_empty() {
            print!("{output}");
        }
    }
}

/// Start the LSP server for editor integration.
fn run_lsp() -> miette::Result<ExitStatus> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| error::CliError::Runtime(err.to_string()))?;
    rt.block_on(starlint_lsp::run_lsp())?;
    Ok(ExitStatus::Success)
}

/// Human-readable label for a rule category.
const fn category_label(category: &starlint_plugin_sdk::rule::Category) -> &'static str {
    use starlint_plugin_sdk::rule::Category;
    match category {
        Category::Correctness => "correctness",
        Category::Style => "style",
        Category::Performance => "performance",
        Category::Suggestion => "suggestion",
        Category::Custom(_) => "custom",
    }
}
