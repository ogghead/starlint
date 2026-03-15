//! Rule: `branches-sharing-code`
//!
//! Flag if/else branches that share identical leading or trailing statements.
//! When the first or last statement of both branches is textually identical,
//! it can be factored out before or after the `if/else` for clarity.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags if/else branches with identical leading or trailing statements.
#[derive(Debug)]
pub struct BranchesSharingCode;

impl LintRule for BranchesSharingCode {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "branches-sharing-code".to_owned(),
            description:
                "Flag if/else branches that share identical leading or trailing statements"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        // Both branches must exist and be block statements.
        let Some(alternate_id) = if_stmt.alternate else {
            return;
        };

        let Some(AstNode::BlockStatement(consequent_block)) = ctx.node(if_stmt.consequent) else {
            return;
        };
        let consequent_body = consequent_block.body.clone();

        let Some(AstNode::BlockStatement(alternate_block)) = ctx.node(alternate_id) else {
            return;
        };
        let alternate_body = alternate_block.body.clone();

        if consequent_body.is_empty() || alternate_body.is_empty() {
            return;
        }

        // Collect diagnostic info first to avoid borrow conflict with `ctx`.
        let diagnostics = {
            let source = ctx.source_text();
            let mut diags: Vec<(Span, &str)> = Vec::new();

            // Check leading (first) statements.
            if let (Some(first_cons), Some(first_alt)) =
                (consequent_body.first(), alternate_body.first())
            {
                if statements_text_equal(*first_cons, *first_alt, source, ctx) {
                    if let Some(n) = ctx.node(*first_cons) {
                        let span = n.span();
                        diags.push((
                            Span::new(span.start, span.end),
                            "This statement appears in both branches and can be moved before the `if`",
                        ));
                    }
                }
            }

            // Check trailing (last) statements.
            // Only report if it is a different statement from the leading one (avoid
            // double-report when both branches have only one statement that is the same).
            if let (Some(last_cons), Some(last_alt)) =
                (consequent_body.last(), alternate_body.last())
            {
                let is_same_as_leading = consequent_body.len() == 1 && alternate_body.len() == 1;
                if !is_same_as_leading && statements_text_equal(*last_cons, *last_alt, source, ctx)
                {
                    if let Some(n) = ctx.node(*last_cons) {
                        let span = n.span();
                        diags.push((
                            Span::new(span.start, span.end),
                            "This statement appears in both branches and can be moved after the `if/else`",
                        ));
                    }
                }
            }

            diags
        };

        for (span, message) in diagnostics {
            ctx.report(Diagnostic {
                rule_name: "branches-sharing-code".to_owned(),
                message: message.to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Compare two statements by their source text.
fn statements_text_equal(a: NodeId, b: NodeId, source: &str, ctx: &LintContext<'_>) -> bool {
    let (Some(a_node), Some(b_node)) = (ctx.node(a), ctx.node(b)) else {
        return false;
    };
    let a_span = a_node.span();
    let b_span = b_node.span();
    let a_start = usize::try_from(a_span.start).unwrap_or(0);
    let a_end = usize::try_from(a_span.end).unwrap_or(0);
    let b_start = usize::try_from(b_span.start).unwrap_or(0);
    let b_end = usize::try_from(b_span.end).unwrap_or(0);

    let Some(a_text) = source.get(a_start..a_end) else {
        return false;
    };
    let Some(b_text) = source.get(b_start..b_end) else {
        return false;
    };

    a_text == b_text
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(BranchesSharingCode);

    #[test]
    fn test_flags_shared_leading_statement() {
        let diags = lint("if (x) { a(); b(); } else { a(); c(); }");
        assert_eq!(
            diags.len(),
            1,
            "shared leading statement a() should be flagged"
        );
    }

    #[test]
    fn test_flags_shared_trailing_statement() {
        let diags = lint("if (x) { a(); b(); } else { c(); b(); }");
        assert_eq!(
            diags.len(),
            1,
            "shared trailing statement b() should be flagged"
        );
    }

    #[test]
    fn test_allows_different_branches() {
        let diags = lint("if (x) { a(); } else { b(); }");
        assert!(diags.is_empty(), "different branches should not be flagged");
    }

    #[test]
    fn test_flags_both_leading_and_trailing() {
        let diags = lint("if (x) { a(); b(); c(); } else { a(); d(); c(); }");
        assert_eq!(
            diags.len(),
            2,
            "both shared leading and trailing statements should be flagged"
        );
    }

    #[test]
    fn test_single_identical_statement_flags_once() {
        let diags = lint("if (x) { a(); } else { a(); }");
        assert_eq!(
            diags.len(),
            1,
            "single identical statement should be flagged once (as leading), not twice"
        );
    }

    #[test]
    fn test_allows_no_alternate() {
        let diags = lint("if (x) { a(); }");
        assert!(diags.is_empty(), "if without else should not be flagged");
    }

    #[test]
    fn test_allows_non_block_branches() {
        let diags = lint("if (x) a(); else b();");
        assert!(diags.is_empty(), "non-block branches should not be flagged");
    }

    #[test]
    fn test_allows_empty_blocks() {
        let diags = lint("if (x) {} else {}");
        assert!(diags.is_empty(), "empty blocks should not be flagged");
    }
}
