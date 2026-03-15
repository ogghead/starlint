//! Rule: `no-self-assign`
//!
//! Disallow assignments where both sides are the same. Self-assignments
//! like `x = x` have no effect and are almost always mistakes.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::AssignmentOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags assignments where the left and right sides are identical.
#[derive(Debug)]
pub struct NoSelfAssign;

impl LintRule for NoSelfAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-self-assign".to_owned(),
            description: "Disallow self-assignment".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Only check plain `=` assignments (not `+=`, `-=`, etc.)
        if assign.operator != AssignmentOperator::Assign {
            return;
        }

        let left_name = assignment_target_name(assign.left, ctx);
        let right_name = expression_name(assign.right, ctx);

        if let (Some(left), Some(right)) = (left_name, right_name) {
            if left == right {
                let stmt_span = Span::new(assign.span.start, assign.span.end);
                let edit = fix_utils::delete_statement(ctx.source_text(), stmt_span);
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
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

/// Extract a simple identifier name from an assignment target node.
fn assignment_target_name<'a>(target_id: NodeId, ctx: &'a LintContext<'_>) -> Option<&'a str> {
    match ctx.node(target_id) {
        Some(AstNode::IdentifierReference(ident)) => Some(ident.name.as_str()),
        Some(AstNode::BindingIdentifier(ident)) => Some(ident.name.as_str()),
        _ => None,
    }
}

/// Extract a simple identifier name from an expression node.
fn expression_name<'a>(expr_id: NodeId, ctx: &'a LintContext<'_>) -> Option<&'a str> {
    match ctx.node(expr_id) {
        Some(AstNode::IdentifierReference(ident)) => Some(ident.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoSelfAssign);

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
