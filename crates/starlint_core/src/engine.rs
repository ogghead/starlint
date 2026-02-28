//! Lint engine: orchestrates parsing, traversal, and diagnostic collection.
//!
//! [`LintSession`] holds the resolved rule set and lints files in parallel.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use rayon::prelude::*;

use crate::diagnostic::OutputFormat;
use crate::error::LintError;
use crate::overrides::OverrideSet;
use crate::parser::{build_semantic, parse_file};
use crate::plugin::PluginHost;
use crate::rule::NativeRule;
use crate::traversal::traverse_and_lint_with_semantic;
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
    /// File-pattern overrides compiled from config.
    override_set: OverrideSet,
    /// Rules loaded but disabled by default (only active via file-pattern overrides).
    disabled_rules: HashSet<String>,
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
            override_set: OverrideSet::empty(),
            disabled_rules: HashSet::new(),
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

    /// Set file-pattern overrides compiled from config.
    #[must_use]
    pub fn with_override_set(mut self, override_set: OverrideSet) -> Self {
        self.override_set = override_set;
        self
    }

    /// Set disabled rules (loaded but suppressed unless overrides activate them).
    #[must_use]
    pub fn with_disabled_rules(mut self, disabled_rules: HashSet<String>) -> Self {
        self.disabled_rules = disabled_rules;
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
                    Err(io_err) => {
                        let err = LintError::FileRead {
                            path: path.display().to_string(),
                            source: io_err,
                        };
                        tracing::warn!("{err}");
                        return Some(FileDiagnostics {
                            path: path.clone(),
                            source_text: String::new(),
                            diagnostics: vec![err.into_diagnostic()],
                        });
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

                // Arena-allocate the program so it can be shared with semantic analysis.
                let program = allocator.alloc(parsed.program);

                // Build semantic data (scope tree, symbol table) if any rule needs it.
                let needs_semantic = self.native_rules.iter().any(|r| r.needs_semantic());
                let semantic = needs_semantic.then(|| build_semantic(program));

                // Native rules via single-pass traversal.
                let mut diags = traverse_and_lint_with_semantic(
                    program,
                    &self.native_rules,
                    source_text,
                    file_path,
                    semantic.as_ref(),
                );

                // External plugin host (WASM, etc.).
                if let Some(host) = &self.plugin_host {
                    let plugin_diags = host.lint_file(file_path, source_text, program);
                    diags.extend(plugin_diags);
                }

                diags
            }
            Err(err) => {
                tracing::warn!("{err}");
                vec![err.into_diagnostic()]
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

        // Apply file-pattern overrides and suppress disabled rules.
        if !self.override_set.is_empty() || !self.disabled_rules.is_empty() {
            self.override_set
                .apply(file_path, &self.disabled_rules, &mut diagnostics);
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

    #[test]
    fn test_lint_single_file_parse_error() {
        let session = LintSession::new(vec![], OutputFormat::Pretty);
        // parse_file returns Err only for unsupported file extensions.
        let result = session.lint_single_file(Path::new("test.py"), "const x = 1;");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.rule_name == "starlint/parse-error"),
            "unsupported file type should produce a synthetic parse-error diagnostic"
        );
    }

    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    #[test]
    fn test_lint_files_parallel() {
        let dir = std::env::temp_dir().join("starlint-test-parallel");
        std::fs::create_dir_all(&dir).ok();

        let file_a = dir.join("a.js");
        let file_b = dir.join("b.js");
        std::fs::write(&file_a, "debugger;").ok();
        // Use a minimal valid statement that shouldn't trigger any rules.
        std::fs::write(&file_b, "'use strict';").ok();

        let rules = crate::rules::all_rules();
        let session = LintSession::new(rules, OutputFormat::Pretty);
        let results = session.lint_files(&[file_a.clone(), file_b.clone()]);

        // File a has debugger statement -> should have diagnostics.
        let a_diags: usize = results
            .iter()
            .filter(|r| r.path == file_a)
            .map(|r| r.diagnostics.len())
            .sum();
        assert!(
            a_diags > 0,
            "file with violations should have diagnostics"
        );

        // Clean up (best-effort).
        std::fs::remove_file(&file_a).ok();
        std::fs::remove_file(&file_b).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_lint_files_io_error() {
        let session = LintSession::new(vec![], OutputFormat::Pretty);
        let nonexistent = PathBuf::from("/nonexistent/starlint-test.js");
        let results = session.lint_files(&[nonexistent]);

        assert_eq!(results.len(), 1, "should return result for unreadable file");
        assert!(
            results.first().is_some_and(|r| r
                .diagnostics
                .iter()
                .any(|d| d.rule_name == "starlint/io-error")),
            "should contain synthetic io-error diagnostic"
        );
    }
}
