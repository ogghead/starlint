//! Rule: `no-instanceof-array` (unicorn)
//!
//! Disallow `instanceof Array`. Use `Array.isArray()` instead, which works
//! across different realms (iframes, workers).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `x instanceof Array`.
#[derive(Debug)]
pub struct NoInstanceofArray;

impl NativeRule for NoInstanceofArray {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-instanceof-array".to_owned(),
            description: "Disallow `instanceof Array` — use `Array.isArray()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != oxc_ast::ast::BinaryOperator::Instanceof {
            return;
        }

        let is_array = matches!(
            &expr.right,
            Expression::Identifier(id) if id.name.as_str() == "Array"
        );

        if is_array {
            ctx.report_warning(
                "no-instanceof-array",
                "Use `Array.isArray()` instead of `instanceof Array`",
                Span::new(expr.span.start, expr.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInstanceofArray)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_instanceof_array() {
        let diags = lint("if (x instanceof Array) {}");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_array_is_array() {
        let diags = lint("if (Array.isArray(x)) {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_instanceof_other() {
        let diags = lint("if (x instanceof Map) {}");
        assert!(diags.is_empty());
    }
}
