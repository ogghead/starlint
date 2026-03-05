//! Rule: `id-length` (eslint)
//!
//! Flag identifiers that are too short. Single-letter variable names
//! (other than `_`) hurt readability and make code harder to search.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default minimum identifier length.
const DEFAULT_MIN: u32 = 2;

/// Flags binding identifiers shorter than the minimum length.
#[derive(Debug)]
pub struct IdLength {
    /// Minimum identifier length.
    min: u32,
}

impl IdLength {
    /// Create a new `IdLength` rule with the default minimum.
    #[must_use]
    pub const fn new() -> Self {
        Self { min: DEFAULT_MIN }
    }
}

impl Default for IdLength {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for IdLength {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "id-length".to_owned(),
            description: "Enforce minimum identifier length".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("min").and_then(serde_json::Value::as_u64) {
            self.min = u32::try_from(n).unwrap_or(DEFAULT_MIN);
        }
        Ok(())
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BindingIdentifier])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BindingIdentifier(id) = kind else {
            return;
        };

        let name = id.name.as_str();

        // Skip the underscore — it's an intentional discard pattern
        if name == "_" {
            return;
        }

        let name_len = u32::try_from(name.len()).unwrap_or(0);
        if name_len < self.min {
            ctx.report(Diagnostic {
                rule_name: "id-length".to_owned(),
                message: format!(
                    "Identifier '{name}' is too short ({name_len} < {}). Use a more descriptive name",
                    self.min
                ),
                span: Span::new(id.span.start, id.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(IdLength::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_short_variable() {
        let diags = lint("let x = 1;");
        assert_eq!(diags.len(), 1, "single-char variable should be flagged");
    }

    #[test]
    fn test_allows_long_variable() {
        let diags = lint("let foo = 1;");
        assert!(
            diags.is_empty(),
            "multi-char variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_underscore() {
        let diags = lint("let _ = 1;");
        assert!(
            diags.is_empty(),
            "underscore should not be flagged (intentional discard)"
        );
    }

    #[test]
    fn test_flags_short_function_name() {
        let diags = lint("function f() {}");
        assert_eq!(
            diags.len(),
            1,
            "single-char function name should be flagged"
        );
    }

    #[test]
    fn test_allows_two_char_name() {
        let diags = lint("let ab = 1;");
        assert!(
            diags.is_empty(),
            "two-char name should not be flagged with default min 2"
        );
    }
}
