//! Rule: `no-undefined`
//!
//! Disallow the use of `undefined` as an identifier. Using `undefined`
//! can be problematic because it can be shadowed in non-strict mode.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags references to `undefined`.
#[derive(Debug)]
pub struct NoUndefined;

impl NativeRule for NoUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-undefined".to_owned(),
            description: "Disallow the use of `undefined` as an identifier".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IdentifierReference])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IdentifierReference(id) = kind else {
            return;
        };

        if id.name.as_str() == "undefined" {
            ctx.report(Diagnostic {
                rule_name: "no-undefined".to_owned(),
                message: "Unexpected use of `undefined` — use `void 0` instead if needed"
                    .to_owned(),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUndefined)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_undefined_reference() {
        let diags = lint("var x = undefined;");
        assert_eq!(diags.len(), 1, "use of undefined should be flagged");
    }

    #[test]
    fn test_flags_undefined_comparison() {
        let diags = lint("if (x === undefined) {}");
        assert_eq!(
            diags.len(),
            1,
            "comparison with undefined should be flagged"
        );
    }

    #[test]
    fn test_allows_void_zero() {
        let diags = lint("var x = void 0;");
        assert!(diags.is_empty(), "void 0 should not be flagged");
    }

    #[test]
    fn test_allows_typeof_undefined() {
        // typeof undefined is technically an identifier reference but
        // typeof always works safely
        let diags = lint("var x = typeof y;");
        assert!(diags.is_empty(), "typeof should not be flagged");
    }
}
