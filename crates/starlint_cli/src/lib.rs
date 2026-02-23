//! Library root for starlint CLI.
//!
//! Orchestrates CLI parsing, config loading, session construction, and linting.

pub mod cli;
pub mod error;

use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Command, OutputFormatArg};
use starlint_core::diagnostic::OutputFormat;
use starlint_core::engine::LintSession;
use starlint_core::file_discovery::discover_files;

/// Run the starlint CLI.
///
/// Parses arguments, loads config, discovers files, lints, and formats output.
pub fn run() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Cli::parse();

    let output_format = match args.format {
        OutputFormatArg::Pretty => OutputFormat::Pretty,
        OutputFormatArg::Json => OutputFormat::Json,
        OutputFormatArg::Compact => OutputFormat::Compact,
    };

    let paths = match &args.command {
        Some(Command::Lint { paths }) | Some(Command::Fix { paths, .. }) => paths.clone(),
        Some(Command::Init) => return run_init(),
        Some(Command::Rules { plugin, json }) => return run_rules(plugin.as_deref(), *json),
        None => args.paths.clone(),
    };

    // Discover files.
    let files = discover_files(&paths);
    if files.is_empty() {
        tracing::warn!("no lintable files found");
        return Ok(());
    }

    tracing::info!("found {} files to lint", files.len());

    // Build lint session (no native rules yet — will be added as rules are implemented).
    let session = LintSession::new(vec![], output_format);

    // Lint files.
    let results = session.lint_files(&files);

    // Format and print results.
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for result in &results {
        let output = starlint_core::diagnostic::format_diagnostics(
            &result.diagnostics,
            &result.source_text,
            &result.path,
            output_format,
        );
        if !output.is_empty() {
            // Use tracing for output since print_stdout is denied.
            tracing::info!("{output}");
        }
        for diag in &result.diagnostics {
            match diag.severity {
                starlint_plugin_sdk::diagnostic::Severity::Error => {
                    total_errors = total_errors.wrapping_add(1);
                }
                starlint_plugin_sdk::diagnostic::Severity::Warning => {
                    total_warnings = total_warnings.wrapping_add(1);
                }
                starlint_plugin_sdk::diagnostic::Severity::Suggestion => {}
            }
        }
    }

    if total_errors > 0 || total_warnings > 0 {
        tracing::info!(
            "found {total_errors} error(s) and {total_warnings} warning(s) in {} file(s)",
            results.len()
        );
    }

    if total_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize a default `starlint.toml` config file.
fn run_init() -> miette::Result<()> {
    let config_path = PathBuf::from("starlint.toml");
    if config_path.exists() {
        tracing::warn!("starlint.toml already exists");
        return Ok(());
    }

    let default_config = r#"# starlint configuration
# See https://github.com/ogghead/starlint for documentation

[settings]
threads = 0  # 0 = auto-detect

# [[plugins]]
# name = "storybook"
# path = "./plugins/starlint-plugin-storybook.wasm"

[rules]
# "no-debugger" = "error"
"#;

    std::fs::write(&config_path, default_config)
        .map_err(|err| miette::miette!("failed to write starlint.toml: {err}"))?;

    tracing::info!("created starlint.toml");
    Ok(())
}

/// List available rules.
fn run_rules(_plugin: Option<&str>, _json: bool) -> miette::Result<()> {
    tracing::info!("no rules registered yet");
    Ok(())
}
