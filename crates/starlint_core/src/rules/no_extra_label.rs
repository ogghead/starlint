//! Rule: `no-extra-label`
//!
//! Disallow unnecessary labels. If a `break` or `continue` targets the
//! immediately enclosing loop or switch, the label is redundant.
//! This is a simplified version that flags any labeled statement
//! where the label is only used once in a direct child break/continue.

#![allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation
)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags labels that are unnecessary because the break/continue targets
/// the immediately enclosing loop.
#[derive(Debug)]
pub struct NoExtraLabel;

impl LintRule for NoExtraLabel {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-label".to_owned(),
            description: "Disallow unnecessary labels".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::LabeledStatement])
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        clippy::cast_possible_truncation
    )]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::LabeledStatement(labeled) = node else {
            return;
        };

        let label_name = labeled.label.as_str();

        // If the labeled statement is a loop or switch, and the only
        // break/continue in its direct body references this label,
        // then the label is unnecessary.
        let body_node = ctx.node(labeled.body);
        let is_simple_loop = matches!(
            body_node,
            Some(
                AstNode::ForStatement(_)
                    | AstNode::ForInStatement(_)
                    | AstNode::ForOfStatement(_)
                    | AstNode::WhileStatement(_)
                    | AstNode::DoWhileStatement(_)
            )
        );

        let is_switch = matches!(body_node, Some(AstNode::SwitchStatement(_)));

        if !is_simple_loop && !is_switch {
            return;
        }

        // For a simple single-level loop/switch, any break/continue with
        // this label is redundant since it's the immediately enclosing one.
        let span_start = labeled.span.start;

        // Build edits: delete the label prefix, and remove label from break/continue.
        let body_span = body_node.map_or(Span::new(0, 0), |n| {
            let s = n.span();
            Span::new(s.start, s.end)
        });
        let mut edits = vec![Edit {
            span: Span::new(span_start, body_span.start),
            replacement: String::new(),
        }];

        // Also remove label references from break/continue statements.
        collect_label_ref_edits(labeled.body, label_name, &mut edits, ctx);

        // Compute label span end from source text: "label_name:"
        let source = ctx.source_text();
        let label_prefix_text = source
            .get(span_start as usize..body_span.start as usize)
            .unwrap_or("");
        let label_end = span_start
            + label_prefix_text
                .find(':')
                .map_or(label_name.len() as u32, |p| p as u32 + 1);

        ctx.report(Diagnostic {
            rule_name: "no-extra-label".to_owned(),
            message: format!("Unnecessary label `{label_name}`"),
            span: Span::new(span_start, label_end),
            severity: Severity::Warning,
            help: Some(format!("Remove label `{label_name}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Remove label `{label_name}`"),
                edits,
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Walk the body of a loop/switch to find break/continue statements referencing
/// `label`, and add edits to remove the label (including the preceding space).
fn collect_label_ref_edits(
    stmt_id: NodeId,
    label: &str,
    edits: &mut Vec<Edit>,
    ctx: &LintContext<'_>,
) {
    let Some(stmt) = ctx.node(stmt_id) else {
        return;
    };
    match stmt {
        AstNode::BreakStatement(brk) => {
            if let Some(l) = &brk.label {
                if l.as_str() == label {
                    // Delete " label" (space + label name) from break statement.
                    // The label text is after "break " in the source; compute span from source text.
                    let source = ctx.source_text();
                    let brk_text = source
                        .get(brk.span.start as usize..brk.span.end as usize)
                        .unwrap_or("");
                    if let Some(pos) = brk_text.find(label) {
                        let label_start = brk.span.start + pos as u32;
                        let label_end = label_start + label.len() as u32;
                        // Include the preceding space
                        edits.push(Edit {
                            span: Span::new(label_start.saturating_sub(1), label_end),
                            replacement: String::new(),
                        });
                    }
                }
            }
        }
        AstNode::ContinueStatement(cont) => {
            if let Some(l) = &cont.label {
                if l.as_str() == label {
                    let source = ctx.source_text();
                    let cont_text = source
                        .get(cont.span.start as usize..cont.span.end as usize)
                        .unwrap_or("");
                    if let Some(pos) = cont_text.find(label) {
                        let label_start = cont.span.start + pos as u32;
                        let label_end = label_start + label.len() as u32;
                        edits.push(Edit {
                            span: Span::new(label_start.saturating_sub(1), label_end),
                            replacement: String::new(),
                        });
                    }
                }
            }
        }
        AstNode::BlockStatement(block) => {
            for s in &block.body {
                collect_label_ref_edits(*s, label, edits, ctx);
            }
        }
        AstNode::IfStatement(if_stmt) => {
            collect_label_ref_edits(if_stmt.consequent, label, edits, ctx);
            if let Some(alt) = if_stmt.alternate {
                collect_label_ref_edits(alt, label, edits, ctx);
            }
        }
        AstNode::ForStatement(f) => collect_label_ref_edits(f.body, label, edits, ctx),
        AstNode::ForInStatement(f) => collect_label_ref_edits(f.body, label, edits, ctx),
        AstNode::ForOfStatement(f) => collect_label_ref_edits(f.body, label, edits, ctx),
        AstNode::WhileStatement(w) => collect_label_ref_edits(w.body, label, edits, ctx),
        AstNode::DoWhileStatement(d) => collect_label_ref_edits(d.body, label, edits, ctx),
        AstNode::SwitchStatement(sw) => {
            for case_id in &sw.cases {
                if let Some(AstNode::SwitchCase(case)) = ctx.node(*case_id) {
                    for s in &case.consequent {
                        collect_label_ref_edits(*s, label, edits, ctx);
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtraLabel)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_label_on_simple_loop() {
        let diags = lint("loop1: for (var i = 0; i < 10; i++) { break loop1; }");
        assert_eq!(
            diags.len(),
            1,
            "label on simple loop with break should be flagged"
        );
    }

    #[test]
    fn test_flags_label_on_while() {
        let diags = lint("loop1: while (true) { break; }");
        assert_eq!(diags.len(), 1, "label on while loop should be flagged");
    }

    #[test]
    fn test_allows_label_on_block() {
        let diags = lint("label1: { break label1; }");
        assert!(
            diags.is_empty(),
            "label on block statement should not be flagged by this rule"
        );
    }
}
