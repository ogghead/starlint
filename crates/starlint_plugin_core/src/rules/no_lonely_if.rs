//! Rule: `no-lonely-if`
//!
//! Disallow `if` as the only statement in an `else` block.
//! `if (a) {} else { if (b) {} }` should be `if (a) {} else if (b) {}`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `else { if (...) {} }` that should be `else if (...) {}`.
#[derive(Debug)]
pub struct NoLonelyIf;

impl LintRule for NoLonelyIf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-lonely-if".to_owned(),
            description: "Disallow `if` as the only statement in an `else` block".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(stmt) = node else {
            return;
        };

        // Check for `else { <single if statement> }`.
        let Some(alt_id) = stmt.alternate else {
            return;
        };

        let Some(AstNode::BlockStatement(block)) = ctx.node(alt_id) else {
            return;
        };

        if block.body.len() != 1 {
            return;
        }

        let Some(first_stmt_id) = block.body.first() else {
            return;
        };

        let Some(AstNode::IfStatement(inner_if)) = ctx.node(*first_stmt_id) else {
            return;
        };

        // Get the inner if-statement source text.
        let inner_start = inner_if.span.start as usize;
        let inner_end = inner_if.span.end as usize;
        let block_span = Span::new(block.span.start, block.span.end);
        let Some(inner_text) = ctx.source_text().get(inner_start..inner_end) else {
            return;
        };

        // Replace the block `{ if (...) {} }` with ` if (...) {}`.
        let replacement = format!(" {inner_text}");

        ctx.report(Diagnostic {
            rule_name: "no-lonely-if".to_owned(),
            message: "Unexpected lonely `if` inside `else` block".to_owned(),
            span: block_span,
            severity: Severity::Warning,
            help: Some("Combine into `else if`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Combine into `else if`".to_owned(),
                edits: vec![Edit {
                    span: block_span,
                    replacement,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoLonelyIf);

    #[test]
    fn test_flags_lonely_if() {
        let diags = lint("if (a) {} else { if (b) {} }");
        assert_eq!(diags.len(), 1, "should flag lonely if in else block");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
    }

    #[test]
    fn test_flags_lonely_if_with_else() {
        let diags = lint("if (a) {} else { if (b) {} else {} }");
        assert_eq!(
            diags.len(),
            1,
            "should flag even when inner if has its own else"
        );
    }

    #[test]
    fn test_ignores_direct_else_if() {
        let diags = lint("if (a) {} else if (b) {}");
        assert!(diags.is_empty(), "direct else-if should not be flagged");
    }

    #[test]
    fn test_ignores_multiple_statements_in_else() {
        let diags = lint("if (a) {} else { console.log(1); if (b) {} }");
        assert!(
            diags.is_empty(),
            "multiple statements in else should not be flagged"
        );
    }

    #[test]
    fn test_ignores_no_alternate() {
        let diags = lint("if (a) {}");
        assert!(diags.is_empty(), "if without else should not be flagged");
    }

    #[test]
    fn test_fix_replaces_block_with_inner_if() {
        let source = "if (a) {} else { if (b) { x(); } }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        let replacement = fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str()));
        assert_eq!(
            replacement,
            Some(" if (b) { x(); }"),
            "fix should replace block with inner if"
        );
    }
}
