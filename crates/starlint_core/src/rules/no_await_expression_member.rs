//! Rule: `no-await-expression-member`
//!
//! Disallow member access on `await` expressions like `(await foo()).bar`.
//! This pattern is error-prone — prefer assigning to a variable first.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags member expressions on `await` expressions.
#[derive(Debug)]
pub struct NoAwaitExpressionMember;

/// Unwrap parenthesized expressions to find the inner expression.
fn unwrap_parens<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut current = expr;
    while let Expression::ParenthesizedExpression(paren) = current {
        current = &paren.expression;
    }
    current
}

impl NativeRule for NoAwaitExpressionMember {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-await-expression-member".to_owned(),
            description: "Disallow member access on `await` expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::StaticMemberExpression(member) => {
                if matches!(
                    unwrap_parens(&member.object),
                    Expression::AwaitExpression(_)
                ) {
                    ctx.report_warning(
                        "no-await-expression-member",
                        "Do not access a member directly on an `await` expression — assign to a variable first",
                        Span::new(member.span.start, member.span.end),
                    );
                }
            }
            AstKind::ComputedMemberExpression(member) => {
                if matches!(
                    unwrap_parens(&member.object),
                    Expression::AwaitExpression(_)
                ) {
                    ctx.report_warning(
                        "no-await-expression-member",
                        "Do not access a member directly on an `await` expression — assign to a variable first",
                        Span::new(member.span.start, member.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAwaitExpressionMember)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_static_member_on_await() {
        let diags = lint("async function f() { (await promise).value; }");
        assert_eq!(diags.len(), 1, "(await promise).value should be flagged");
    }

    #[test]
    fn test_flags_computed_member_on_await() {
        let diags = lint("async function f() { (await promise)[0]; }");
        assert_eq!(diags.len(), 1, "(await promise)[0] should be flagged");
    }

    #[test]
    fn test_allows_variable_then_member() {
        let diags = lint("async function f() { const val = await promise; val.value; }");
        assert!(
            diags.is_empty(),
            "accessing member on a variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_bare_await() {
        let diags = lint("async function f() { await promise; }");
        assert!(diags.is_empty(), "bare await should not be flagged");
    }
}
