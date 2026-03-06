//! Rule: `no-cond-assign`
//!
//! Disallow assignment operators in conditional expressions. Using `=` instead
//! of `==` or `===` in conditions is a common mistake: `if (x = 5)` assigns 5
//! to x instead of comparing.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags assignment expressions used directly in conditions.
#[derive(Debug)]
pub struct NoCondAssign;

impl NativeRule for NoCondAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-cond-assign".to_owned(),
            description: "Disallow assignment operators in conditional expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ConditionalExpression,
            AstType::DoWhileStatement,
            AstType::ForStatement,
            AstType::IfStatement,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::IfStatement(if_stmt) => {
                check_condition(&if_stmt.test, ctx);
            }
            AstKind::WhileStatement(while_stmt) => {
                check_condition(&while_stmt.test, ctx);
            }
            AstKind::DoWhileStatement(do_while) => {
                check_condition(&do_while.test, ctx);
            }
            AstKind::ForStatement(for_stmt) => {
                if let Some(test) = &for_stmt.test {
                    check_condition(test, ctx);
                }
            }
            AstKind::ConditionalExpression(cond) => {
                check_condition(&cond.test, ctx);
            }
            _ => {}
        }
    }
}

/// Check if an expression (used as a condition) is an assignment.
fn check_condition(expr: &Expression<'_>, ctx: &mut NativeLintContext<'_>) {
    if let Expression::AssignmentExpression(assign) = expr {
        // Fix: replace `=` with `===`
        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let left_span = assign.left.span();
            let right_span = assign.right.span();
            let left_text = source
                .get(left_span.start as usize..left_span.end as usize)
                .unwrap_or("");
            let right_text = source
                .get(right_span.start as usize..right_span.end as usize)
                .unwrap_or("");
            let replacement = format!("{left_text} === {right_text}");
            Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(assign.span.start, assign.span.end),
                    replacement,
                }],
                is_snippet: false,
            })
        };

        ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
            rule_name: "no-cond-assign".to_owned(),
            message: "Unexpected assignment in conditional expression".to_owned(),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Error,
            help: Some("Did you mean `===` instead of `=`?".to_owned()),
            fix,
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCondAssign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_assignment_in_if() {
        let diags = lint("if (x = 5) {}");
        assert_eq!(diags.len(), 1, "assignment in if should be flagged");
    }

    #[test]
    fn test_flags_assignment_in_while() {
        let diags = lint("while (x = 5) {}");
        assert_eq!(diags.len(), 1, "assignment in while should be flagged");
    }

    #[test]
    fn test_flags_assignment_in_do_while() {
        let diags = lint("do {} while (x = 5);");
        assert_eq!(diags.len(), 1, "assignment in do-while should be flagged");
    }

    #[test]
    fn test_flags_assignment_in_for() {
        let diags = lint("for (;x = 5;) {}");
        assert_eq!(
            diags.len(),
            1,
            "assignment in for condition should be flagged"
        );
    }

    #[test]
    fn test_allows_parenthesized_assignment_in_ternary() {
        // Parenthesized assignments are intentional — consistent with ESLint's
        // default "except-parens" mode
        let diags = lint("var y = (x = 5) ? 1 : 2;");
        assert!(
            diags.is_empty(),
            "parenthesized assignment in ternary should not be flagged"
        );
    }

    #[test]
    fn test_allows_comparison() {
        let diags = lint("if (x === 5) {}");
        assert!(diags.is_empty(), "comparison should not be flagged");
    }

    #[test]
    fn test_allows_assignment_outside_condition() {
        let diags = lint("var x = 5;");
        assert!(
            diags.is_empty(),
            "assignment outside condition should not be flagged"
        );
    }
}
