//! Rule: `bad-char-at-comparison` (OXC)
//!
//! Catch comparisons of `.charAt()` result against a multi-character string.
//! `.charAt()` always returns a single character (or empty string), so
//! comparing it to a multi-character string is always false.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.charAt()` compared to a multi-character string.
#[derive(Debug)]
pub struct BadCharAtComparison;

impl LintRule for BadCharAtComparison {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-char-at-comparison".to_owned(),
            description: "Catch `.charAt()` compared to a multi-character string".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        if !expr.operator.is_equality() {
            return;
        }

        // Check left.charAt() == "multi-char" or "multi-char" == right.charAt()
        let flagged = (is_char_at_call(ctx, expr.left) && is_multi_char_string(ctx, expr.right))
            || (is_multi_char_string(ctx, expr.left) && is_char_at_call(ctx, expr.right));

        if flagged {
            ctx.report(Diagnostic {
                rule_name: "bad-char-at-comparison".to_owned(),
                message: "`.charAt()` returns a single character — comparing to a multi-character \
                 string is always false"
                    .to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `false`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: "false".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if a node is a `.charAt()` call.
fn is_char_at_call(ctx: &LintContext<'_>, id: NodeId) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(id) else {
        return false;
    };
    matches!(
        ctx.node(call.callee),
        Some(AstNode::StaticMemberExpression(member)) if member.property == "charAt"
    )
}

/// Check if a node is a string literal with more than one character.
fn is_multi_char_string(ctx: &LintContext<'_>, id: NodeId) -> bool {
    matches!(ctx.node(id), Some(AstNode::StringLiteral(s)) if s.value.len() > 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(BadCharAtComparison);

    #[test]
    fn test_flags_char_at_vs_multi_char() {
        let diags = lint("if (s.charAt(0) === 'ab') {}");
        assert_eq!(
            diags.len(),
            1,
            "charAt compared to multi-char string should be flagged"
        );
    }

    #[test]
    fn test_flags_reverse_order() {
        let diags = lint("if ('ab' === s.charAt(0)) {}");
        assert_eq!(
            diags.len(),
            1,
            "multi-char string compared to charAt should be flagged"
        );
    }

    #[test]
    fn test_allows_single_char_comparison() {
        let diags = lint("if (s.charAt(0) === 'a') {}");
        assert!(
            diags.is_empty(),
            "charAt compared to single char should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_char_at_call() {
        let diags = lint("if (s.indexOf('ab') === 'ab') {}");
        assert!(diags.is_empty(), "non-charAt call should not be flagged");
    }
}
