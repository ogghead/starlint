//! Lint engine: orchestrates parsing, traversal, and diagnostic collection.
//!
//! [`LintSession`] holds the resolved rule set and lints files in parallel.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use rayon::prelude::*;

use crate::diagnostic::OutputFormat;
use crate::parser::parse_file;
use crate::plugin::PluginHost;
use crate::rule::NativeRule;
use crate::traversal::traverse_and_lint;
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity};

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
/// Holds the set of active rules, optional plugin host, and severity
/// overrides from config. Lints files in parallel.
pub struct LintSession {
    /// Active native rules.
    native_rules: Vec<Box<dyn NativeRule>>,
    /// Optional external plugin host (e.g., WASM).
    plugin_host: Option<Box<dyn PluginHost>>,
    /// Output format.
    output_format: OutputFormat,
    /// Severity overrides from config (rule name → configured severity).
    severity_overrides: HashMap<String, Severity>,
}

impl LintSession {
    /// Create a new lint session with the given rules.
    #[must_use]
    pub fn new(native_rules: Vec<Box<dyn NativeRule>>, output_format: OutputFormat) -> Self {
        Self {
            native_rules,
            plugin_host: None,
            output_format,
            severity_overrides: HashMap::new(),
        }
    }

    /// Set severity overrides from config.
    #[must_use]
    pub fn with_severity_overrides(mut self, overrides: HashMap<String, Severity>) -> Self {
        self.severity_overrides = overrides;
        self
    }

    /// Set the plugin host for external plugins (WASM, etc.).
    #[must_use]
    pub fn with_plugin_host(mut self, host: Box<dyn PluginHost>) -> Self {
        self.plugin_host = Some(host);
        self
    }

    /// Lint multiple files in parallel.
    ///
    /// Returns per-file diagnostics for files that had issues.
    pub fn lint_files(&self, files: &[PathBuf]) -> Vec<FileDiagnostics> {
        files
            .par_iter()
            .filter_map(|path| {
                let source_text = match std::fs::read_to_string(path) {
                    Ok(text) => text,
                    Err(err) => {
                        tracing::warn!("failed to read {}: {err}", path.display());
                        return None;
                    }
                };
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

        let mut diagnostics = match parse_result {
            Ok(parsed) => {
                if parsed.panicked {
                    tracing::warn!("parse errors in {}", file_path.display());
                }

                // Native rules via single-pass traversal.
                let mut diags =
                    traverse_and_lint(&parsed.program, &self.native_rules, source_text, file_path);

                // External plugin host (WASM, etc.).
                if let Some(host) = &self.plugin_host {
                    let plugin_diags = host.lint_file(file_path, source_text, &parsed.program);
                    diags.extend(plugin_diags);
                }

                diags
            }
            Err(err) => {
                tracing::warn!("failed to parse {}: {err}", file_path.display());
                Vec::new()
            }
        };

        // Apply severity overrides from config.
        if !self.severity_overrides.is_empty() {
            for diag in &mut diagnostics {
                if let Some(severity) = self.severity_overrides.get(&diag.rule_name) {
                    diag.severity = *severity;
                }
            }
        }

        FileDiagnostics {
            path: file_path.to_path_buf(),
            source_text: source_text.to_owned(),
            diagnostics,
        }
    }

    /// Get the configured output format.
    #[must_use]
    pub const fn output_format(&self) -> OutputFormat {
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
