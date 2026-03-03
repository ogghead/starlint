//! Rule: `no-array-constructor`
//!
//! Disallow `Array` constructors. Use array literal syntax `[]` instead.
//! `new Array(1, 2)` should be `[1, 2]`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Array()` and `new Array()` with multiple arguments.
#[derive(Debug)]
pub struct NoArrayConstructor;

impl NativeRule for NoArrayConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-constructor".to_owned(),
            description: "Disallow `Array` constructor".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::NewExpression(new_expr) => {
                if matches!(&new_expr.callee, Expression::Identifier(id) if id.name.as_str() == "Array")
                    && new_expr.arguments.len() != 1
                {
                    ctx.report_warning(
                        "no-array-constructor",
                        "Use array literal `[]` instead of `Array` constructor",
                        Span::new(new_expr.span.start, new_expr.span.end),
                    );
                }
            }
            AstKind::CallExpression(call) => {
                if matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "Array")
                    && call.arguments.len() != 1
                {
                    ctx.report_warning(
                        "no-array-constructor",
                        "Use array literal `[]` instead of `Array` constructor",
                        Span::new(call.span.start, call.span.end),
                    );
                }
            }
            _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayConstructor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_array_multiple() {
        let diags = lint("var a = new Array(1, 2, 3);");
        assert_eq!(diags.len(), 1, "new Array(1, 2, 3) should be flagged");
    }

    #[test]
    fn test_flags_array_call_empty() {
        let diags = lint("var a = Array();");
        assert_eq!(diags.len(), 1, "Array() empty should be flagged");
    }

    #[test]
    fn test_allows_single_arg() {
        let diags = lint("var a = new Array(5);");
        assert!(
            diags.is_empty(),
            "new Array(5) creates sparse array — should not be flagged"
        );
    }

    #[test]
    fn test_allows_array_literal() {
        let diags = lint("var a = [1, 2, 3];");
        assert!(diags.is_empty(), "array literal should not be flagged");
    }
}
