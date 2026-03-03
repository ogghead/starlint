//! Rule: `prefer-date-now`
//!
//! Prefer `Date.now()` over `new Date().getTime()` and `+new Date()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Date().getTime()` and `+new Date()` — prefer `Date.now()`.
#[derive(Debug)]
pub struct PreferDateNow;

/// Check if an expression is `new Date()` (with zero arguments).
fn is_new_date_no_args(expr: &Expression<'_>) -> bool {
    let Expression::NewExpression(new_expr) = expr else {
        return false;
    };
    let Expression::Identifier(id) = &new_expr.callee else {
        return false;
    };
    id.name.as_str() == "Date" && new_expr.arguments.is_empty()
}

impl NativeRule for PreferDateNow {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-date-now".to_owned(),
            description: "Prefer `Date.now()` over `new Date().getTime()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // new Date().getTime()
            AstKind::CallExpression(call) => {
                let Expression::StaticMemberExpression(member) = &call.callee else {
                    return;
                };
                if member.property.name.as_str() != "getTime" {
                    return;
                }
                if !call.arguments.is_empty() {
                    return;
                }
                if !is_new_date_no_args(&member.object) {
                    return;
                }

                ctx.report_warning(
                    "prefer-date-now",
                    "Use `Date.now()` instead of `new Date().getTime()`",
                    Span::new(call.span.start, call.span.end),
                );
            }
            // +new Date()
            AstKind::UnaryExpression(unary) => {
                if unary.operator != oxc_ast::ast::UnaryOperator::UnaryPlus {
                    return;
                }
                if !is_new_date_no_args(&unary.argument) {
                    return;
                }

                ctx.report_warning(
                    "prefer-date-now",
                    "Use `Date.now()` instead of `+new Date()`",
                    Span::new(unary.span.start, unary.span.end),
                );
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDateNow)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_date_get_time() {
        let diags = lint("const t = new Date().getTime();");
        assert_eq!(diags.len(), 1, "should flag new Date().getTime()");
    }

    #[test]
    fn test_flags_plus_new_date() {
        let diags = lint("const t = +new Date();");
        assert_eq!(diags.len(), 1, "should flag +new Date()");
    }

    #[test]
    fn test_allows_date_now() {
        let diags = lint("const t = Date.now();");
        assert!(diags.is_empty(), "Date.now() should not be flagged");
    }

    #[test]
    fn test_allows_new_date_with_args() {
        let diags = lint("const t = new Date(2024).getTime();");
        assert!(
            diags.is_empty(),
            "new Date(arg).getTime() should not be flagged"
        );
    }
}
