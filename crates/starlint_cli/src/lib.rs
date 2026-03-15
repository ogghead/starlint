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
use starlint_config::resolve::{load_config, resolve_config};
use starlint_core::diagnostic::{self, OutputFormat};
use starlint_core::engine::{FileDiagnostics, LintSession};
use starlint_core::file_discovery::discover_files;
use starlint_loader::all_rule_metas;
use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::FixKind;
use starlint_rule_framework::apply_fixes;

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
        OutputFormatArg::Github => OutputFormat::Github,
        OutputFormatArg::Gitlab => OutputFormat::Gitlab,
        OutputFormatArg::Junit => OutputFormat::Junit,
        OutputFormatArg::Sarif => OutputFormat::Sarif,
        OutputFormatArg::Stylish => OutputFormat::Stylish,
    };

    // Determine fix mode from command or flags.
    let (paths, fix_enabled, fix_dangerous, fix_dry_run) = match &args.command {
        Some(Command::Fix {
            paths, dangerous, ..
        }) => (paths.clone(), true, *dangerous, false),
        Some(Command::Lint { paths }) => (
            paths.clone(),
            args.fix,
            args.fix_dangerous,
            args.fix_dry_run,
        ),
        Some(Command::Lsp) => return run_lsp(),
        Some(Command::Init) => {
            run_init()?;
            return Ok(ExitStatus::Success);
        }
        Some(Command::Rules { plugin, json }) => {
            run_rules(plugin.as_deref(), *json);
            return Ok(ExitStatus::Success);
        }
        None => (
            args.paths.clone(),
            args.fix,
            args.fix_dangerous,
            args.fix_dry_run,
        ),
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

    // Optionally load result cache.
    let mut cache = args
        .cache
        .then(|| starlint_core::cache::LintCache::load(&args.cache_location));

    // Filter files using cache (skip unchanged files).
    let (files_to_lint, _cached_counts) = if let Some(ref c) = cache {
        let (need_lint, counts) = c.filter_unchanged(&files);
        tracing::debug!(
            "cache: {} cached, {} need linting",
            files.len().saturating_sub(need_lint.len()),
            need_lint.len()
        );
        (need_lint, counts)
    } else {
        (files.clone(), starlint_core::cache::CachedCounts::default())
    };

    // Lint files.
    let lint_start = Instant::now();
    let results = session.lint_files(&files_to_lint);
    let lint_elapsed = lint_start.elapsed();

    // Update cache with new results.
    if let Some(ref mut c) = cache {
        for result in &results {
            let mut errors = 0u32;
            let mut warnings = 0u32;
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
            cache_update_file(c, &result.path, &result.source_text, errors, warnings);
        }
        if let Err(err) = c.save(&args.cache_location) {
            tracing::warn!("failed to save cache: {err}");
        }
    }

    if fix_enabled {
        apply_fixes_to_files(&results, fix_dangerous, &session);
    } else if fix_dry_run {
        report_dry_run_fixes(&results, fix_dangerous);
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
    Ok(resolve_config(Path::new("."))?)
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

        if diagnostic::is_document_format(output_format) {
            // Document formats produce a single output from all files.
            write_document_diagnostics(&mut writer, results, output_format);
        } else {
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

/// Write all diagnostics as a single document for formats that require it.
///
/// GitLab Code Quality, `JUnit` XML, and SARIF are document-level formats:
/// their output wraps all diagnostics in a single structure rather than
/// streaming per-file.
fn write_document_diagnostics(
    writer: &mut impl IoWrite,
    results: &[FileDiagnostics],
    output_format: OutputFormat,
) {
    let collected: Vec<_> = results
        .iter()
        .map(|r| (r.diagnostics.clone(), r.source_text.clone(), r.path.clone()))
        .collect();

    #[allow(clippy::let_underscore_must_use)]
    let _ = match output_format {
        OutputFormat::Gitlab => diagnostic::write_document_gitlab(writer, &collected),
        OutputFormat::Junit => diagnostic::write_document_junit(writer, &collected),
        OutputFormat::Sarif => diagnostic::write_document_sarif(writer, &collected),
        _ => Ok(()),
    };
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

/// Update a single file entry in the lint cache.
fn cache_update_file(
    cache: &mut starlint_core::cache::LintCache,
    path: &Path,
    source_text: &str,
    errors: u32,
    warnings: u32,
) {
    // Re-read the file to get the actual on-disk content for hashing,
    // but if source_text is available (non-empty), use it as-is.
    if source_text.is_empty() {
        if let Ok(content) = std::fs::read_to_string(path) {
            cache.update(path, &content, errors, warnings);
        }
    } else {
        cache.update(path, source_text, errors, warnings);
    }
}

/// Report fixes that would be applied in dry-run mode without writing to disk.
#[allow(clippy::print_stderr)]
fn report_dry_run_fixes(results: &[FileDiagnostics], include_dangerous: bool) {
    let mut total_fixes = 0usize;

    for result in results {
        let fixable = filter_fixable_diags(&result.diagnostics, include_dangerous);
        if !fixable.is_empty() {
            let count = fixable.len();
            total_fixes = total_fixes.saturating_add(count);
            eprintln!("{}: {} fix(es) available", result.path.display(), count,);
        }
    }

    if total_fixes > 0 {
        eprintln!("{total_fixes} fix(es) would be applied (dry run)");
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use starlint_core::engine::FileDiagnostics;
    use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Span};
    use starlint_plugin_sdk::rule::{Category, FixKind};

    /// Helper: create a minimal diagnostic with the given severity and no fix.
    fn make_diag(severity: Severity) -> Diagnostic {
        Diagnostic {
            rule_name: String::from("test/rule"),
            message: String::from("test message"),
            span: Span::new(0, 1),
            severity,
            help: None,
            fix: None,
            labels: Vec::new(),
        }
    }

    /// Helper: create a diagnostic with a fix of the given kind.
    fn make_diag_with_fix(severity: Severity, fix_kind: FixKind) -> Diagnostic {
        Diagnostic {
            rule_name: String::from("test/fixable"),
            message: String::from("fixable issue"),
            span: Span::new(0, 1),
            severity,
            help: None,
            fix: Some(Fix {
                kind: fix_kind,
                message: String::from("apply fix"),
                edits: Vec::new(),
                is_snippet: false,
            }),
            labels: Vec::new(),
        }
    }

    // ── category_label ──────────────────────────────────────────────────

    #[test]
    fn category_label_correctness() {
        assert_eq!(category_label(&Category::Correctness), "correctness");
    }

    #[test]
    fn category_label_style() {
        assert_eq!(category_label(&Category::Style), "style");
    }

    #[test]
    fn category_label_performance() {
        assert_eq!(category_label(&Category::Performance), "performance");
    }

    #[test]
    fn category_label_suggestion() {
        assert_eq!(category_label(&Category::Suggestion), "suggestion");
    }

    #[test]
    fn category_label_custom() {
        assert_eq!(
            category_label(&Category::Custom(String::from("my-cat"))),
            "custom"
        );
    }

    // ── ExitStatus ──────────────────────────────────────────────────────

    #[test]
    fn exit_status_eq() {
        assert_eq!(ExitStatus::Success, ExitStatus::Success);
        assert_eq!(ExitStatus::LintErrors, ExitStatus::LintErrors);
        assert_ne!(ExitStatus::Success, ExitStatus::LintErrors);
    }

    #[test]
    fn exit_status_debug() {
        let success_fmt = format!("{:?}", ExitStatus::Success);
        assert!(success_fmt.contains("Success"));

        let errors_fmt = format!("{:?}", ExitStatus::LintErrors);
        assert!(errors_fmt.contains("LintErrors"));
    }

    #[test]
    fn exit_status_copy() {
        let s = ExitStatus::Success;
        let copied = s;
        assert_eq!(s, copied);
    }

    // ── filter_fixable_diags ────────────────────────────────────────────

    #[test]
    fn filter_fixable_empty_input() {
        let result = filter_fixable_diags(&[], false);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_fixable_no_fix_diagnostics() {
        let diags = vec![make_diag(Severity::Error), make_diag(Severity::Warning)];
        let result = filter_fixable_diags(&diags, false);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_fixable_safe_fix_always_included() {
        let diags = vec![make_diag_with_fix(Severity::Error, FixKind::SafeFix)];
        let safe_only = filter_fixable_diags(&diags, false);
        assert_eq!(safe_only.len(), 1);

        let with_dangerous = filter_fixable_diags(&diags, true);
        assert_eq!(with_dangerous.len(), 1);
    }

    #[test]
    fn filter_fixable_suggestion_fix_excluded_without_dangerous() {
        let diags = vec![make_diag_with_fix(
            Severity::Warning,
            FixKind::SuggestionFix,
        )];
        let safe_only = filter_fixable_diags(&diags, false);
        assert!(safe_only.is_empty());

        let with_dangerous = filter_fixable_diags(&diags, true);
        assert_eq!(with_dangerous.len(), 1);
    }

    #[test]
    fn filter_fixable_dangerous_fix_excluded_without_dangerous() {
        let diags = vec![make_diag_with_fix(Severity::Warning, FixKind::DangerousFix)];
        let safe_only = filter_fixable_diags(&diags, false);
        assert!(safe_only.is_empty());

        let with_dangerous = filter_fixable_diags(&diags, true);
        assert_eq!(with_dangerous.len(), 1);
    }

    #[test]
    fn filter_fixable_none_fix_kind_excluded() {
        let diags = vec![make_diag_with_fix(Severity::Error, FixKind::None)];
        let result = filter_fixable_diags(&diags, true);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_fixable_mixed_diagnostics() {
        let diags = vec![
            make_diag(Severity::Error),
            make_diag_with_fix(Severity::Error, FixKind::SafeFix),
            make_diag_with_fix(Severity::Warning, FixKind::DangerousFix),
            make_diag_with_fix(Severity::Warning, FixKind::None),
        ];

        let safe_only = filter_fixable_diags(&diags, false);
        assert_eq!(safe_only.len(), 1);
        assert_eq!(
            safe_only.first().map(|d| &d.rule_name),
            Some(&String::from("test/fixable"))
        );

        let with_dangerous = filter_fixable_diags(&diags, true);
        assert_eq!(with_dangerous.len(), 2);
    }

    // ── report_diagnostics ──────────────────────────────────────────────

    #[test]
    fn report_diagnostics_empty() {
        let results: Vec<FileDiagnostics> = Vec::new();
        let counts = report_diagnostics(&results, OutputFormat::Count);
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.warnings, 0);
    }

    #[test]
    fn report_diagnostics_counts_errors_and_warnings() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("test.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![
                make_diag(Severity::Error),
                make_diag(Severity::Error),
                make_diag(Severity::Warning),
            ],
        }];
        // Use Count mode to skip formatting output.
        let counts = report_diagnostics(&results, OutputFormat::Count);
        assert_eq!(counts.errors, 2);
        assert_eq!(counts.warnings, 1);
    }

    #[test]
    fn report_diagnostics_ignores_suggestions() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("test.js"),
            source_text: String::from("x"),
            diagnostics: vec![make_diag(Severity::Suggestion)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Count);
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.warnings, 0);
    }

    #[test]
    fn report_diagnostics_multiple_files() {
        let results = vec![
            FileDiagnostics {
                path: PathBuf::from("a.js"),
                source_text: String::from("a"),
                diagnostics: vec![make_diag(Severity::Error)],
            },
            FileDiagnostics {
                path: PathBuf::from("b.js"),
                source_text: String::from("b"),
                diagnostics: vec![make_diag(Severity::Warning), make_diag(Severity::Warning)],
            },
        ];
        let counts = report_diagnostics(&results, OutputFormat::Count);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 2);
    }

    // ── configure_thread_pool ───────────────────────────────────────────

    #[test]
    fn configure_thread_pool_zero_does_not_panic() {
        // Both values zero means "do nothing" — should not panic.
        configure_thread_pool(0, 0);
    }

    // ── load_merged_config ──────────────────────────────────────────────

    #[test]
    fn load_merged_config_nonexistent_explicit_path_errors() {
        let result = load_merged_config(Some(Path::new("/tmp/nonexistent-starlint-config.toml")));
        assert!(result.is_err());
    }

    #[test]
    fn load_merged_config_none_returns_default() {
        // With no explicit path and no starlint.toml in cwd, should resolve to defaults.
        let result = load_merged_config(None);
        assert!(result.is_ok());
    }

    // ── MAX_FIX_PASSES constant ─────────────────────────────────────────

    #[test]
    fn max_fix_passes_is_reasonable() {
        const { assert!(MAX_FIX_PASSES >= 2) };
        const { assert!(MAX_FIX_PASSES <= 100) };
    }

    // ── write_atomic ──────────────────────────────────────────────────

    #[test]
    fn test_write_atomic_creates_and_renames() {
        let dir = PathBuf::from("/tmp/starlint-test-write-atomic");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join("output.js");
        let content = "console.log('hello');";
        let result = write_atomic(&dir, &target, content);
        assert!(result.is_ok());
        let read_back = std::fs::read_to_string(&target).unwrap_or_default();
        assert_eq!(read_back, content);
        // Cleanup
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── cache_update_file ─────────────────────────────────────────────

    #[test]
    fn test_cache_update_file_with_source_text() {
        let mut cache = starlint_core::cache::LintCache::new();
        let path = PathBuf::from("test_source.js");
        cache_update_file(&mut cache, &path, "var x = 1;", 1, 0);
        // The cache should have been updated (no panic, no error).
        // We can verify by checking that a second update with different content
        // still succeeds.
        cache_update_file(&mut cache, &path, "var y = 2;", 0, 1);
    }

    #[test]
    fn test_cache_update_file_empty_source_reads_file() {
        // When source_text is empty, cache_update_file tries to read the file
        // from disk. If the file doesn't exist, the cache entry is simply skipped.
        let mut cache = starlint_core::cache::LintCache::new();
        let path = PathBuf::from("/tmp/starlint-test-nonexistent-file.js");
        // Should not panic even when the file doesn't exist.
        cache_update_file(&mut cache, &path, "", 0, 0);

        // Now test with an actual file on disk.
        let dir = PathBuf::from("/tmp/starlint-test-cache-empty-src");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::create_dir_all(&dir);
        let real_path = dir.join("real.js");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::write(&real_path, "let a = 1;");
        cache_update_file(&mut cache, &real_path, "", 1, 0);
        // Cleanup
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── report_dry_run_fixes ──────────────────────────────────────────

    #[test]
    #[allow(clippy::print_stderr)]
    fn test_report_dry_run_fixes_with_fixable() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("fix_me.js"),
            source_text: String::from("debugger;"),
            diagnostics: vec![make_diag_with_fix(Severity::Warning, FixKind::SafeFix)],
        }];
        // Should not panic; output goes to stderr.
        report_dry_run_fixes(&results, false);
    }

    #[test]
    fn test_report_dry_run_fixes_empty() {
        let results: Vec<FileDiagnostics> = Vec::new();
        report_dry_run_fixes(&results, false);
        // No panic, no output — just verifying it handles empty input.
    }

    // ── configure_thread_pool ─────────────────────────────────────────

    #[test]
    fn test_configure_thread_pool_cli_takes_priority() {
        // CLI threads > 0 should attempt to configure the pool.
        // This may warn if the global pool is already initialized, but must not panic.
        configure_thread_pool(2, 0);
    }

    // ── report_diagnostics ────────────────────────────────────────────

    #[test]
    fn test_report_diagnostics_with_pretty_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("pretty.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![
                make_diag(Severity::Error),
                make_diag(Severity::Warning),
                make_diag(Severity::Warning),
            ],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Pretty);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 2);
    }

    // ── write_document_diagnostics ──────────────────────────────────

    #[test]
    fn test_write_document_diagnostics_gitlab() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("doc.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let mut buf = Vec::new();
        write_document_diagnostics(&mut buf, &results, OutputFormat::Gitlab);
        let output = String::from_utf8_lossy(&buf);
        // GitLab Code Quality outputs JSON array.
        assert!(
            output.starts_with('['),
            "expected JSON array, got: {output}"
        );
    }

    #[test]
    fn test_write_document_diagnostics_junit() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("doc.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Warning)],
        }];
        let mut buf = Vec::new();
        write_document_diagnostics(&mut buf, &results, OutputFormat::Junit);
        let output = String::from_utf8_lossy(&buf);
        assert!(
            output.contains("testsuites"),
            "expected JUnit XML, got: {output}"
        );
    }

    #[test]
    fn test_write_document_diagnostics_sarif() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("doc.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let mut buf = Vec::new();
        write_document_diagnostics(&mut buf, &results, OutputFormat::Sarif);
        let output = String::from_utf8_lossy(&buf);
        assert!(
            output.contains("sarif"),
            "expected SARIF JSON, got: {output}"
        );
    }

    #[test]
    fn test_write_document_diagnostics_non_document_format_is_noop() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("doc.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let mut buf = Vec::new();
        write_document_diagnostics(&mut buf, &results, OutputFormat::Pretty);
        assert!(
            buf.is_empty(),
            "non-document format should produce no output"
        );
    }

    // ── report_diagnostics with document formats ────────────────────

    #[test]
    fn test_report_diagnostics_with_gitlab_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("gl.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error), make_diag(Severity::Warning)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Gitlab);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 1);
    }

    #[test]
    fn test_report_diagnostics_with_junit_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("ju.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Junit);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 0);
    }

    #[test]
    fn test_report_diagnostics_with_sarif_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("sa.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Warning)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Sarif);
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.warnings, 1);
    }

    #[test]
    fn test_report_diagnostics_with_github_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("gh.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Github);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 0);
    }

    #[test]
    fn test_report_diagnostics_with_stylish_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("sty.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Warning), make_diag(Severity::Warning)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Stylish);
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.warnings, 2);
    }

    #[test]
    fn test_report_diagnostics_with_compact_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("cmp.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Compact);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 0);
    }

    #[test]
    fn test_report_diagnostics_with_json_format() {
        let results = vec![FileDiagnostics {
            path: PathBuf::from("js.js"),
            source_text: String::from("var x = 1;"),
            diagnostics: vec![make_diag(Severity::Error)],
        }];
        let counts = report_diagnostics(&results, OutputFormat::Json);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 0);
    }

    // ── category_label all variants ───────────────────────────────────

    #[test]
    fn test_category_label_all_variants() {
        let variants: Vec<Category> = vec![
            Category::Correctness,
            Category::Style,
            Category::Performance,
            Category::Suggestion,
            Category::Custom(String::from("test")),
        ];
        for variant in &variants {
            let label = category_label(variant);
            assert!(!label.is_empty(), "category label should be non-empty");
        }
    }
}
