//! Rule: `no-self-assign`
//!
//! Disallow assignments where both sides are the same. Self-assignments
//! like `x = x` have no effect and are almost always mistakes.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags assignments where the left and right sides are identical.
#[derive(Debug)]
pub struct NoSelfAssign;

impl NativeRule for NoSelfAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-self-assign".to_owned(),
            description: "Disallow self-assignment".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Only check plain `=` assignments (not `+=`, `-=`, etc.)
        if assign.operator != AssignmentOperator::Assign {
            return;
        }

        let left_name = assignment_target_name(&assign.left);
        let right_name = expression_name(&assign.right);

        if let (Some(left), Some(right)) = (left_name, right_name) {
            if left == right {
                let stmt_span = Span::new(assign.span.start, assign.span.end);
                let edit = fix_utils::delete_statement(ctx.source_text(), stmt_span);
                let fix = Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove this self-assignment".to_owned(),
                    edits: vec![edit],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-self-assign".to_owned(),
                    message: format!("`{left}` is assigned to itself"),
                    span: stmt_span,
                    severity: Severity::Error,
                    help: Some("Remove this self-assignment".to_owned()),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

/// Extract a simple identifier name from an assignment target.
fn assignment_target_name<'a>(target: &'a AssignmentTarget<'a>) -> Option<&'a str> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

/// Extract a simple identifier name from an expression.
fn expression_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        _ => None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSelfAssign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_self_assign() {
        let diags = lint("x = x;");
        assert_eq!(diags.len(), 1, "x = x should be flagged");
    }

    #[test]
    fn test_allows_different_vars() {
        let diags = lint("x = y;");
        assert!(diags.is_empty(), "x = y should not be flagged");
    }

    #[test]
    fn test_allows_compound_assignment() {
        let diags = lint("x += x;");
        assert!(
            diags.is_empty(),
            "compound assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_member_expressions() {
        // Member expressions like `a.b = a.b` are not checked (would need
        // deeper comparison logic).
        let diags = lint("a.b = a.b;");
        assert!(diags.is_empty(), "member self-assign not checked yet");
    }
}
