//! Rule: `no-unused-labels`
//!
//! Disallow unused labels. Labels that are not referenced by any `break` or
//! `continue` statement are likely mistakes and add unnecessary complexity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags labeled statements where the label is never used by break/continue.
#[derive(Debug)]
pub struct NoUnusedLabels;

impl LintRule for NoUnusedLabels {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-labels".to_owned(),
            description: "Disallow unused labels".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::LabeledStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::LabeledStatement(labeled) = node else {
            return;
        };

        let label_name = labeled.label.clone();

        // Check if the label is referenced in the body
        let body_node = ctx.node(labeled.body);
        if !body_node.is_some_and(|n| statement_references_label(ctx, n, &label_name)) {
            let label_span = labeled.span;
            // Delete from the start of the labeled statement to the start of the body.
            let body_start = ctx
                .node(labeled.body)
                .map_or(labeled.span.end, |n| n.span().start);
            let delete_span = Span::new(label_span.start, body_start);

            ctx.report(Diagnostic {
                rule_name: "no-unused-labels".to_owned(),
                message: format!("Label `{label_name}` is defined but never used"),
                span: Span::new(label_span.start, label_span.end),
                severity: Severity::Error,
                help: Some(format!("Remove label `{label_name}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Remove label `{label_name}`"),
                    edits: vec![Edit {
                        span: delete_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a statement (or its children) contains a break/continue that
/// references the given label.
fn statement_references_label(ctx: &LintContext<'_>, stmt: &AstNode, label: &str) -> bool {
    match stmt {
        AstNode::BreakStatement(brk) => brk.label.as_deref() == Some(label),
        AstNode::ContinueStatement(cont) => cont.label.as_deref() == Some(label),
        AstNode::BlockStatement(block) => block.body.iter().any(|&id| {
            ctx.node(id)
                .is_some_and(|s| statement_references_label(ctx, s, label))
        }),
        AstNode::IfStatement(if_stmt) => {
            let cons = ctx
                .node(if_stmt.consequent)
                .is_some_and(|n| statement_references_label(ctx, n, label));
            let alt = if_stmt
                .alternate
                .and_then(|id| ctx.node(id))
                .is_some_and(|n| statement_references_label(ctx, n, label));
            cons || alt
        }
        AstNode::WhileStatement(while_stmt) => ctx
            .node(while_stmt.body)
            .is_some_and(|n| statement_references_label(ctx, n, label)),
        AstNode::DoWhileStatement(do_while) => ctx
            .node(do_while.body)
            .is_some_and(|n| statement_references_label(ctx, n, label)),
        AstNode::ForStatement(for_stmt) => ctx
            .node(for_stmt.body)
            .is_some_and(|n| statement_references_label(ctx, n, label)),
        AstNode::ForInStatement(for_in) => ctx
            .node(for_in.body)
            .is_some_and(|n| statement_references_label(ctx, n, label)),
        AstNode::ForOfStatement(for_of) => ctx
            .node(for_of.body)
            .is_some_and(|n| statement_references_label(ctx, n, label)),
        AstNode::SwitchStatement(switch) => switch.cases.iter().any(|&case_id| {
            ctx.node(case_id).is_some_and(|case_node| {
                if let AstNode::SwitchCase(case) = case_node {
                    case.consequent.iter().any(|&s_id| {
                        ctx.node(s_id)
                            .is_some_and(|s| statement_references_label(ctx, s, label))
                    })
                } else {
                    false
                }
            })
        }),
        AstNode::TryStatement(try_stmt) => {
            let block_has = ctx.node(try_stmt.block).is_some_and(|n| {
                if let AstNode::BlockStatement(block) = n {
                    block.body.iter().any(|&id| {
                        ctx.node(id)
                            .is_some_and(|s| statement_references_label(ctx, s, label))
                    })
                } else {
                    false
                }
            });
            let handler_has = try_stmt
                .handler
                .and_then(|id| ctx.node(id))
                .is_some_and(|n| {
                    if let AstNode::CatchClause(catch) = n {
                        ctx.node(catch.body).is_some_and(|body_node| {
                            if let AstNode::BlockStatement(block) = body_node {
                                block.body.iter().any(|&id| {
                                    ctx.node(id)
                                        .is_some_and(|s| statement_references_label(ctx, s, label))
                                })
                            } else {
                                false
                            }
                        })
                    } else {
                        false
                    }
                });
            let fin_has = try_stmt
                .finalizer
                .and_then(|id| ctx.node(id))
                .is_some_and(|n| {
                    if let AstNode::BlockStatement(block) = n {
                        block.body.iter().any(|&id| {
                            ctx.node(id)
                                .is_some_and(|s| statement_references_label(ctx, s, label))
                        })
                    } else {
                        false
                    }
                });
            block_has || handler_has || fin_has
        }
        AstNode::LabeledStatement(labeled) => ctx
            .node(labeled.body)
            .is_some_and(|n| statement_references_label(ctx, n, label)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnusedLabels)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_unused_label() {
        let diags = lint("A: var foo = 0;");
        assert_eq!(diags.len(), 1, "unused label A should be flagged");
    }

    #[test]
    fn test_flags_unused_loop_label() {
        let diags = lint("B: for (var i = 0; i < 10; i++) { break; }");
        assert_eq!(
            diags.len(),
            1,
            "label B with unlabeled break should be flagged"
        );
    }

    #[test]
    fn test_allows_used_label_break() {
        let diags = lint("A: for (var i = 0; i < 10; i++) { break A; }");
        assert!(
            diags.is_empty(),
            "label A used in break should not be flagged"
        );
    }

    #[test]
    fn test_allows_used_label_continue() {
        let diags = lint("A: for (var i = 0; i < 10; i++) { continue A; }");
        assert!(
            diags.is_empty(),
            "label A used in continue should not be flagged"
        );
    }

    #[test]
    fn test_allows_nested_label_usage() {
        let diags = lint("A: for (;;) { for (;;) { break A; } }");
        assert!(
            diags.is_empty(),
            "label A used in nested break should not be flagged"
        );
    }

    #[test]
    fn test_allows_label_used_in_do_while() {
        let diags = lint("A: do { break A; } while (true);");
        assert!(
            diags.is_empty(),
            "label A used in break inside do-while should not be flagged"
        );
    }

    #[test]
    fn test_allows_label_used_in_for_in() {
        let diags = lint("A: for (let x in obj) { break A; }");
        assert!(
            diags.is_empty(),
            "label A used in break inside for-in should not be flagged"
        );
    }

    #[test]
    fn test_allows_label_used_in_for_of() {
        let diags = lint("A: for (let x of arr) { break A; }");
        assert!(
            diags.is_empty(),
            "label A used in break inside for-of should not be flagged"
        );
    }

    #[test]
    fn test_allows_label_used_in_switch() {
        let diags = lint("A: switch(x) { case 1: break A; }");
        assert!(
            diags.is_empty(),
            "label A used in break inside switch should not be flagged"
        );
    }

    #[test]
    fn test_allows_label_used_in_try() {
        let diags = lint("A: try { break A; } catch(e) {}");
        assert!(
            diags.is_empty(),
            "label A used in break inside try should not be flagged"
        );
    }

    #[test]
    fn test_flags_unused_label_do_while() {
        let diags = lint("A: do { break; } while (true);");
        assert_eq!(
            diags.len(),
            1,
            "label A with unlabeled break in do-while should be flagged"
        );
    }

    #[test]
    fn test_flags_unused_label_switch() {
        let diags = lint("A: switch(x) { case 1: break; }");
        assert_eq!(
            diags.len(),
            1,
            "label A with unlabeled break in switch should be flagged"
        );
    }
}
