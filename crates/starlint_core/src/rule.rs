//! Native rule trait and lint context.
//!
//! Native rules operate directly on oxc AST types for maximum performance.
//! Each rule receives an [`AstKind`] variant during traversal and can emit
//! diagnostics via the [`NativeLintContext`].

use std::fmt::Debug;
use std::path::Path;

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::RuleMeta;

/// Trait implemented by native lint rules.
///
/// Rules receive AST nodes during single-pass traversal and emit diagnostics.
/// Implement [`run`](NativeRule::run) for per-node checks or
/// [`run_once`](NativeRule::run_once) for file-level checks.
pub trait NativeRule: Debug + Send + Sync {
    /// Metadata describing this rule.
    fn meta(&self) -> RuleMeta;

    /// Called for each AST node during traversal.
    ///
    /// Default implementation does nothing. Override to inspect specific node kinds.
    fn run(&self, _kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {}

    /// Called once per file, after traversal completes.
    ///
    /// Use for file-level checks (e.g. "file must have a default export").
    fn run_once(&self, _ctx: &mut NativeLintContext<'_>) {}

    /// Whether this rule needs per-node traversal.
    ///
    /// Return `false` if the rule only implements [`run_once`](NativeRule::run_once).
    fn needs_traversal(&self) -> bool {
        true
    }

    /// Configure this rule from a JSON config value.
    ///
    /// Called during session setup when the config contains options for this rule.
    fn configure(&mut self, _config: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}

/// Context provided to native rules during linting.
///
/// Provides access to source text, file path, and a method to report diagnostics.
pub struct NativeLintContext<'a> {
    /// Original source text.
    source_text: &'a str,
    /// Path of the file being linted.
    file_path: &'a Path,
    /// Accumulated diagnostics.
    diagnostics: Vec<Diagnostic>,
}

impl<'a> NativeLintContext<'a> {
    /// Create a new lint context.
    pub const fn new(source_text: &'a str, file_path: &'a Path) -> Self {
        Self {
            source_text,
            file_path,
            diagnostics: Vec::new(),
        }
    }

    /// Get the source text of the file being linted.
    #[must_use]
    pub const fn source_text(&self) -> &str {
        self.source_text
    }

    /// Get the file path.
    #[must_use]
    pub const fn file_path(&self) -> &Path {
        self.file_path
    }

    /// Report a diagnostic.
    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Report a simple error diagnostic.
    pub fn report_error(&mut self, rule_name: &str, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic {
            rule_name: rule_name.to_owned(),
            message: message.to_owned(),
            span,
            severity: Severity::Error,
            help: None,
            fix: None,
            labels: vec![],
        });
    }

    /// Report a simple warning diagnostic.
    pub fn report_warning(&mut self, rule_name: &str, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic {
            rule_name: rule_name.to_owned(),
            message: message.to_owned(),
            span,
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }

    /// Consume the context and return collected diagnostics.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_lint_context_report() {
        let mut ctx = NativeLintContext::new("let x = 1;", Path::new("test.ts"));
        ctx.report_error("test/rule", "bad code", Span::new(0, 3));
        let diags = ctx.into_diagnostics();
        assert_eq!(diags.len(), 1, "should have one diagnostic");
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("test/rule"),
            "rule name should match"
        );
    }

    #[test]
    fn test_lint_context_source_text() {
        let ctx = NativeLintContext::new("const a = 1;", Path::new("test.js"));
        assert_eq!(
            ctx.source_text(),
            "const a = 1;",
            "source text should match"
        );
    }
}
