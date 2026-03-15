//! Rule: `prefer-ternary`
//!
//! Prefer ternary expressions over simple `if`/`else` that both return or
//! both assign to the same variable. Ternary expressions are more concise
//! for these trivial patterns.

#![allow(clippy::indexing_slicing)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::AssignmentOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags simple `if`/`else` blocks that could be ternary expressions.
#[derive(Debug)]
pub struct PreferTernary;

impl LintRule for PreferTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-ternary".to_owned(),
            description: "Prefer ternary expressions over simple if/else assignments or returns"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    #[allow(clippy::as_conversions)]
    #[allow(clippy::indexing_slicing, clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        // Must have an else branch
        let Some(alt_id) = if_stmt.alternate else {
            return;
        };

        let consequent = unwrap_single_statement(if_stmt.consequent, ctx);
        let alternate = unwrap_single_statement(alt_id, ctx);

        let (Some(cons_id), Some(alt_id)) = (consequent, alternate) else {
            return;
        };

        let cons_node = ctx.node(cons_id);
        let alt_node = ctx.node(alt_id);

        // Case 1: both branches are single return statements with arguments
        let both_return = matches!(
            (&cons_node, &alt_node),
            (Some(AstNode::ReturnStatement(c)), Some(AstNode::ReturnStatement(a)))
            if c.argument.is_some() && a.argument.is_some()
        );

        // Case 2: both branches are single assignment expressions to the same
        // variable with the plain `=` operator
        let both_assign_same = is_simple_assign(cons_id, ctx)
            .zip(is_simple_assign(alt_id, ctx))
            .is_some_and(|(left, right)| left == right);

        if !both_return && !both_assign_same {
            return;
        }

        let source = ctx.source_text();
        let test_span = ctx.node(if_stmt.test).map(starlint_ast::AstNode::span);
        let (cond_start, cond_end) = match test_span {
            Some(s) => (s.start as usize, s.end as usize),
            None => return,
        };
        let cond_text = source.get(cond_start..cond_end).unwrap_or("");

        let fix = if both_return {
            build_return_ternary(source, cond_text, cons_id, alt_id, ctx)
        } else {
            build_assign_ternary(source, cond_text, cons_id, alt_id, ctx)
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-ternary".to_owned(),
            message: "This `if`/`else` can be replaced with a ternary expression".to_owned(),
            span: Span::new(if_stmt.span.start, if_stmt.span.end),
            severity: Severity::Warning,
            help: Some("Use a ternary expression".to_owned()),
            fix: fix.map(|replacement| Fix {
                kind: FixKind::SuggestionFix,
                message: "Convert to ternary expression".to_owned(),
                edits: vec![Edit {
                    span: Span::new(if_stmt.span.start, if_stmt.span.end),
                    replacement,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// If the statement is a block with exactly one statement, return that
/// statement's `NodeId`. If it is already a non-block statement, return it directly.
/// Returns `None` for blocks with zero or multiple statements.
fn unwrap_single_statement(stmt_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    match ctx.node(stmt_id)? {
        AstNode::BlockStatement(block) => (block.body.len() == 1).then(|| block.body[0]),
        _ => Some(stmt_id),
    }
}

/// If the statement is an expression statement containing a plain `=`
/// assignment, return the assignment target name. Returns `None` otherwise.
fn is_simple_assign(stmt_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let AstNode::ExpressionStatement(expr_stmt) = ctx.node(stmt_id)? else {
        return None;
    };
    let AstNode::AssignmentExpression(assign) = ctx.node(expr_stmt.expression)? else {
        return None;
    };
    if assign.operator != AssignmentOperator::Assign {
        return None;
    }
    assignment_target_name(assign.left, ctx)
}

/// Extract a simple identifier name from an assignment target.
fn assignment_target_name(target_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(target_id)? {
        AstNode::IdentifierReference(ident) => Some(ident.name.clone()),
        _ => None,
    }
}

/// Build `return cond ? cons_val : alt_val;`
#[allow(clippy::as_conversions)]
fn build_return_ternary(
    source: &str,
    cond_text: &str,
    cons_id: NodeId,
    alt_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let AstNode::ReturnStatement(cons_ret) = ctx.node(cons_id)? else {
        return None;
    };
    let AstNode::ReturnStatement(alt_ret) = ctx.node(alt_id)? else {
        return None;
    };
    let cons_arg_id = cons_ret.argument?;
    let alt_arg_id = alt_ret.argument?;

    let cons_span = ctx.node(cons_arg_id)?.span();
    let alt_span = ctx.node(alt_arg_id)?.span();

    let if_val = source.get(cons_span.start as usize..cons_span.end as usize)?;
    let else_val = source.get(alt_span.start as usize..alt_span.end as usize)?;

    Some(format!("return {cond_text} ? {if_val} : {else_val};"))
}

/// Build `target = cond ? cons_val : alt_val;`
#[allow(clippy::as_conversions)]
fn build_assign_ternary(
    source: &str,
    cond_text: &str,
    if_stmt_id: NodeId,
    else_stmt_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let AstNode::ExpressionStatement(if_expr) = ctx.node(if_stmt_id)? else {
        return None;
    };
    let AstNode::AssignmentExpression(if_assign) = ctx.node(if_expr.expression)? else {
        return None;
    };
    let AstNode::ExpressionStatement(else_expr) = ctx.node(else_stmt_id)? else {
        return None;
    };
    let AstNode::AssignmentExpression(else_assign) = ctx.node(else_expr.expression)? else {
        return None;
    };

    let target_name = assignment_target_name(if_assign.left, ctx)?;

    let if_right_span = ctx.node(if_assign.right)?.span();
    let else_right_span = ctx.node(else_assign.right)?.span();

    let if_val = source.get(if_right_span.start as usize..if_right_span.end as usize)?;
    let else_val = source.get(else_right_span.start as usize..else_right_span.end as usize)?;

    Some(format!(
        "{target_name} = {cond_text} ? {if_val} : {else_val};"
    ))
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferTernary);

    #[test]
    fn test_flags_simple_return() {
        let diags = lint("function f(x) { if (x) { return a; } else { return b; } }");
        assert_eq!(diags.len(), 1, "simple if/else return should be flagged");
    }

    #[test]
    fn test_allows_no_else() {
        let diags = lint("function f(x) { if (x) { return a; } }");
        assert!(diags.is_empty(), "if without else should not be flagged");
    }

    #[test]
    fn test_allows_multiple_statements_in_consequent() {
        let diags = lint("function f(x) { if (x) { foo(); return a; } else { return b; } }");
        assert!(
            diags.is_empty(),
            "multiple statements in if-block should not be flagged"
        );
    }

    #[test]
    fn test_flags_simple_assignment() {
        let diags = lint("var a; if (x) { a = 1; } else { a = 2; }");
        assert_eq!(
            diags.len(),
            1,
            "simple if/else assignment to same var should be flagged"
        );
    }

    #[test]
    fn test_allows_different_assignment_targets() {
        let diags = lint("if (x) { a = 1; } else { b = 2; }");
        assert!(
            diags.is_empty(),
            "assignment to different vars should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("function f(x) { if (x) { return; } else { return; } }");
        assert!(
            diags.is_empty(),
            "empty returns should not be flagged (no value to ternary-ize)"
        );
    }
}
