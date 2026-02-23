//! Lint engine: orchestrates parsing, traversal, and diagnostic collection.
//!
//! [`LintSession`] holds the resolved rule set and lints files in parallel.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use rayon::prelude::*;

use crate::diagnostic::OutputFormat;
use crate::parser::parse_file;
use crate::rule::NativeRule;
use crate::traversal::traverse_and_lint;
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Diagnostics collected for a single file.
#[derive(Debug, Clone)]
pub struct FileDiagnostics {
    /// File path.
    pub path: PathBuf,
    /// Original source text.
    pub source_text: String,
    /// Diagnostics found.
    pub diagnostics: Vec<Diagnostic>,
}

/// A configured lint session.
///
/// Holds the set of active rules and can lint files in parallel.
pub struct LintSession {
    /// Active native rules.
    native_rules: Vec<Box<dyn NativeRule>>,
    /// Output format.
    output_format: OutputFormat,
}

impl LintSession {
    /// Create a new lint session with the given rules.
    #[must_use]
    pub fn new(native_rules: Vec<Box<dyn NativeRule>>, output_format: OutputFormat) -> Self {
        Self {
            native_rules,
            output_format,
        }
    }

    /// Lint multiple files in parallel.
    ///
    /// Returns per-file diagnostics for files that had issues.
    pub fn lint_files(&self, files: &[PathBuf]) -> Vec<FileDiagnostics> {
        files
            .par_iter()
            .filter_map(|path| {
                let source_text = std::fs::read_to_string(path).ok()?;
                let result = self.lint_single_file(path, &source_text);
                if result.diagnostics.is_empty() {
                    None
                } else {
                    Some(result)
                }
            })
            .collect()
    }

    /// Lint a single file.
    #[must_use]
    pub fn lint_single_file(&self, file_path: &Path, source_text: &str) -> FileDiagnostics {
        let allocator = Allocator::default();
        let parse_result = parse_file(&allocator, source_text, file_path);

        let diagnostics = match parse_result {
            Ok(parsed) => {
                if parsed.panicked {
                    tracing::warn!("parse errors in {}", file_path.display());
                }
                traverse_and_lint(
                    &parsed.program,
                    &self.native_rules,
                    source_text,
                    file_path,
                )
            }
            Err(err) => {
                tracing::warn!("failed to parse {}: {err}", file_path.display());
                Vec::new()
            }
        };

        FileDiagnostics {
            path: file_path.to_path_buf(),
            source_text: source_text.to_owned(),
            diagnostics,
        }
    }

    /// Get the configured output format.
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_lint_session_no_rules() {
        let session = LintSession::new(vec![], OutputFormat::Pretty);
        let result = session.lint_single_file(Path::new("test.js"), "debugger;");
        assert!(
            result.diagnostics.is_empty(),
            "no rules should produce no diagnostics"
        );
    }
}
