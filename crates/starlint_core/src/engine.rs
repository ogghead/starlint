//! Lint engine: orchestrates parsing, traversal, and diagnostic collection.
//!
//! [`LintSession`] holds the resolved plugin set and lints files in parallel.
//! All rule providers — native Rust rules and WASM plugins alike — implement
//! the [`Plugin`] trait.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use starlint_parser::ParseOptions;
use starlint_rule_framework::{FileContext, Plugin};

use crate::diagnostic::OutputFormat;
use crate::error::LintError;
use crate::overrides::OverrideSet;
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
/// Holds the set of active plugins and severity overrides from config.
/// Lints files in parallel.
pub struct LintSession {
    /// All active plugins (native rule bundles and WASM plugins alike).
    plugins: Vec<Box<dyn Plugin>>,
    /// Output format.
    output_format: OutputFormat,
    /// Severity overrides from config (rule name → configured severity).
    severity_overrides: HashMap<String, Severity>,
    /// File-pattern overrides compiled from config.
    override_set: OverrideSet,
    /// Rules loaded but disabled by default (only active via file-pattern overrides).
    disabled_rules: HashSet<String>,
    /// Pre-computed: whether any plugin needs scope analysis.
    needs_scope_analysis: bool,
}

impl LintSession {
    /// Create a new lint session from a set of plugins.
    #[must_use]
    pub fn new(plugins: Vec<Box<dyn Plugin>>, output_format: OutputFormat) -> Self {
        let needs_scope_analysis = plugins.iter().any(|p| p.needs_scope_analysis());
        Self {
            plugins,
            output_format,
            severity_overrides: HashMap::new(),
            override_set: OverrideSet::empty(),
            disabled_rules: HashSet::new(),
            needs_scope_analysis,
        }
    }

    /// Set severity overrides from config.
    #[must_use]
    pub fn with_severity_overrides(mut self, overrides: HashMap<String, Severity>) -> Self {
        self.severity_overrides = overrides;
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
        // Validate file extension before parsing.
        if !Self::is_supported_extension(file_path) {
            let err = LintError::Parse {
                path: file_path.display().to_string(),
            };
            return FileDiagnostics {
                path: file_path.to_path_buf(),
                source_text: String::new(),
                diagnostics: vec![err.into_diagnostic()],
            };
        }

        // Parse with the custom parser directly into AstTree.
        let options = ParseOptions::from_path(file_path);
        let parse_result = starlint_parser::parse(source_text, options);

        if parse_result.panicked {
            tracing::warn!("parse errors in {}", file_path.display());
        }

        let tree = parse_result.tree;

        // Build scope analysis if any plugin needs it.
        let scope_data = self
            .needs_scope_analysis
            .then(|| starlint_scope::build_scope_data(&tree));

        let extension = file_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let ctx = FileContext {
            file_path,
            source_text,
            extension,
            tree: &tree,
            scope_data: scope_data.as_ref(),
        };

        // Dispatch to all plugins uniformly.
        let mut diagnostics: Vec<Diagnostic> = self
            .plugins
            .iter()
            .flat_map(|plugin| plugin.lint_file(&ctx))
            .collect();

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
            // Only clone source text when there are diagnostics (needed for line/col formatting).
            source_text: if diagnostics.is_empty() {
                String::new()
            } else {
                source_text.to_owned()
            },
            diagnostics,
        }
    }

    /// Get the configured output format.
    #[must_use]
    pub const fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Check if a file has a supported JS/TS extension.
    fn is_supported_extension(file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| {
                matches!(
                    ext,
                    "js" | "mjs" | "cjs" | "jsx" | "mjsx" | "ts" | "mts" | "cts" | "tsx" | "mtsx"
                )
            })
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use starlint_rule_framework::LintRulePlugin;

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
    fn test_lint_session_empty_plugin() {
        let plugin: Box<dyn Plugin> = Box::new(LintRulePlugin::new(vec![]));
        let session = LintSession::new(vec![plugin], OutputFormat::Pretty);
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

        let plugin: Box<dyn Plugin> = starlint_plugin_core::create_plugin();
        let session = LintSession::new(vec![plugin], OutputFormat::Pretty);
        let results = session.lint_files(&[file_a.clone(), file_b.clone()]);

        // File a has debugger statement -> should have diagnostics.
        let a_diags: usize = results
            .iter()
            .filter(|r| r.path == file_a)
            .map(|r| r.diagnostics.len())
            .sum();
        assert!(a_diags > 0, "file with violations should have diagnostics");

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

    #[test]
    fn test_severity_overrides() {
        let plugin: Box<dyn Plugin> = starlint_plugin_core::create_plugin();
        let mut overrides = HashMap::new();
        overrides.insert("no-debugger".to_owned(), Severity::Error);
        let session =
            LintSession::new(vec![plugin], OutputFormat::Pretty).with_severity_overrides(overrides);

        let result = session.lint_single_file(Path::new("test.js"), "debugger;");
        // The no-debugger rule should fire and have its severity overridden to Error.
        let debugger_diags: Vec<&Diagnostic> = result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == "no-debugger")
            .collect();
        assert!(
            !debugger_diags.is_empty(),
            "no-debugger rule should produce diagnostics"
        );
        assert!(
            debugger_diags.iter().all(|d| d.severity == Severity::Error),
            "severity should be overridden to Error"
        );
    }

    #[test]
    fn test_disabled_rules() {
        let plugin: Box<dyn Plugin> = starlint_plugin_core::create_plugin();
        let mut disabled = HashSet::new();
        disabled.insert("no-debugger".to_owned());
        let session =
            LintSession::new(vec![plugin], OutputFormat::Pretty).with_disabled_rules(disabled);

        let result = session.lint_single_file(Path::new("test.js"), "debugger;");
        let has_debugger = result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "no-debugger");
        assert!(
            !has_debugger,
            "disabled rule should not produce diagnostics"
        );
    }

    #[test]
    fn test_output_format_getter() {
        let session_pretty = LintSession::new(vec![], OutputFormat::Pretty);
        assert_eq!(session_pretty.output_format(), OutputFormat::Pretty);

        let session_json = LintSession::new(vec![], OutputFormat::Json);
        assert_eq!(session_json.output_format(), OutputFormat::Json);

        let session_compact = LintSession::new(vec![], OutputFormat::Compact);
        assert_eq!(session_compact.output_format(), OutputFormat::Compact);

        let session_count = LintSession::new(vec![], OutputFormat::Count);
        assert_eq!(session_count.output_format(), OutputFormat::Count);
    }

    #[test]
    fn test_is_supported_extension_valid() {
        let supported = [
            "js", "mjs", "cjs", "jsx", "mjsx", "ts", "mts", "cts", "tsx", "mtsx",
        ];
        for ext in &supported {
            let path = PathBuf::from(format!("file.{ext}"));
            let session = LintSession::new(vec![], OutputFormat::Pretty);
            let result = session.lint_single_file(&path, "const x = 1;");
            assert!(
                !result
                    .diagnostics
                    .iter()
                    .any(|d| d.rule_name == "starlint/parse-error"),
                "extension .{ext} should be supported"
            );
        }
    }

    #[test]
    fn test_is_supported_extension_unsupported() {
        let unsupported = ["py", "rs", "css"];
        for ext in &unsupported {
            let path = PathBuf::from(format!("file.{ext}"));
            let session = LintSession::new(vec![], OutputFormat::Pretty);
            let result = session.lint_single_file(&path, "const x = 1;");
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|d| d.rule_name == "starlint/parse-error"),
                "extension .{ext} should produce parse-error"
            );
        }
    }

    #[test]
    fn test_lint_single_file_with_real_rules() {
        let plugin: Box<dyn Plugin> = starlint_plugin_core::create_plugin();
        let session = LintSession::new(vec![plugin], OutputFormat::Pretty);
        let result = session.lint_single_file(Path::new("test.js"), "debugger;");

        assert!(
            !result.diagnostics.is_empty(),
            "debugger statement should trigger diagnostics"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.rule_name == "no-debugger"),
            "should include no-debugger diagnostic"
        );
        // Source text should be preserved when diagnostics are present.
        assert_eq!(result.source_text, "debugger;");
    }

    #[test]
    fn test_scope_analysis_path() {
        // The core plugin has rules that need scope analysis (e.g. no-unused-vars).
        let plugin: Box<dyn Plugin> = starlint_plugin_core::create_plugin();
        assert!(
            plugin.needs_scope_analysis(),
            "core plugin should need scope analysis"
        );
        let session = LintSession::new(vec![plugin], OutputFormat::Pretty);
        // Lint code that exercises scope analysis without crashing.
        let result = session.lint_single_file(Path::new("test.js"), "const x = 1; console.log(x);");
        // Should not crash; diagnostics may or may not be present.
        // Just verify the path didn't panic and returned a valid result.
        assert_eq!(result.path, Path::new("test.js"));
    }
}
