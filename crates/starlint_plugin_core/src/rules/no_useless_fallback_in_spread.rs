//! Rule: `no-useless-fallback-in-spread` (unicorn)
//!
//! Disallow useless fallback when spreading in object literals.
//! `{...(obj || {})}` and `{...(obj ?? {})}` are unnecessary because
//! spreading `undefined`/`null` in object literals is a no-op.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::LogicalOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `{...(obj || {})}` and `{...(obj ?? {})}` patterns.
#[derive(Debug)]
pub struct NoUselessFallbackInSpread;

impl LintRule for NoUselessFallbackInSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-fallback-in-spread".to_owned(),
            description: "Disallow useless fallback in spread".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::SpreadElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::SpreadElement(spread) = node else {
            return;
        };

        // Check for `(obj || {})` or `(obj ?? {})` in spread
        let Some(arg_node) = ctx.node(spread.argument) else {
            return;
        };

        let AstNode::LogicalExpression(logical) = arg_node else {
            return;
        };

        // Must be `||` or `??`
        if !matches!(
            logical.operator,
            LogicalOperator::Or | LogicalOperator::Coalesce
        ) {
            return;
        }

        // Right side must be an empty object `{}`
        let Some(AstNode::ObjectExpression(obj)) = ctx.node(logical.right) else {
            return;
        };

        if obj.properties.is_empty() {
            // Replace the spread argument with just the left-hand side.
            let left_span = ctx.node(logical.left).map_or(
                starlint_ast::types::Span::new(0, 0),
                starlint_ast::AstNode::span,
            );
            let left_text = ctx
                .source_text()
                .get(
                    usize::try_from(left_span.start).unwrap_or(0)
                        ..usize::try_from(left_span.end).unwrap_or(0),
                )
                .unwrap_or("")
                .to_owned();
            // Replace the spread argument (everything after `...`)
            let arg_span = arg_node.span();
            let arg_span_val = Span::new(arg_span.start, arg_span.end);
            ctx.report(Diagnostic {
                rule_name: "no-useless-fallback-in-spread".to_owned(),
                message: "The empty object fallback in spread is unnecessary; spreading `undefined`/`null` is a no-op".to_owned(),
                span: Span::new(spread.span.start, spread.span.end),
                severity: Severity::Warning,
                help: Some("Remove the fallback `|| {}`/`?? {}`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove the empty object fallback".to_owned(),
                    edits: vec![Edit {
                        span: arg_span_val,
                        replacement: left_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUselessFallbackInSpread);

    #[test]
    fn test_flags_or_empty_object() {
        let diags = lint("var x = {...(obj || {})};");
        assert_eq!(diags.len(), 1, "...(obj || {{}}) should be flagged");
    }

    #[test]
    fn test_flags_coalesce_empty_object() {
        let diags = lint("var x = {...(obj ?? {})};");
        assert_eq!(diags.len(), 1, "...(obj ?? {{}}) should be flagged");
    }

    #[test]
    fn test_allows_spread_without_fallback() {
        let diags = lint("var x = {...obj};");
        assert!(
            diags.is_empty(),
            "spread without fallback should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_empty_fallback() {
        let diags = lint("var x = {...(obj || { a: 1 })};");
        assert!(diags.is_empty(), "non-empty fallback should not be flagged");
    }
}
