//! CLI argument definitions using clap derive.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// starlint — a fast JS/TS linter with WASM plugin support.
#[derive(Debug, Parser)]
#[command(name = "starlint", version, about)]
pub struct Cli {
    /// Subcommand (default: lint).
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Paths to lint (files or directories).
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Output format.
    #[arg(long, short, default_value = "pretty", value_enum)]
    pub format: OutputFormatArg,

    /// Path to config file (auto-discovered if omitted).
    #[arg(long, short)]
    pub config: Option<PathBuf>,

    /// Apply safe fixes.
    #[arg(long)]
    pub fix: bool,

    /// Also apply dangerous fixes.
    #[arg(long, requires = "fix")]
    pub fix_dangerous: bool,

    /// Maximum number of warnings before failing (0 = unlimited).
    #[arg(long, default_value = "0")]
    pub max_warnings: usize,

    /// Number of threads (0 = auto-detect).
    #[arg(long, default_value = "0")]
    pub threads: usize,

    /// Skip WASM plugins.
    #[arg(long)]
    pub no_plugins: bool,

    /// Print timing information.
    #[arg(long)]
    pub timing: bool,
}

/// CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Lint files (default behavior).
    Lint {
        /// Paths to lint.
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Apply fixes to files.
    Fix {
        /// Paths to fix.
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        /// Also apply dangerous fixes.
        #[arg(long)]
        dangerous: bool,
    },
    /// Initialize a starlint.toml config file.
    Init,
    /// List available rules.
    Rules {
        /// Filter by plugin name.
        #[arg(long)]
        plugin: Option<String>,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
}

/// Output format argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormatArg {
    /// Human-readable colored output.
    Pretty,
    /// JSON output.
    Json,
    /// Compact single-line output.
    Compact,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn test_cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
