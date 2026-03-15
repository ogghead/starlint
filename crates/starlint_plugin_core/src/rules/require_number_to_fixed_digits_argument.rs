//! Rule: `require-number-to-fixed-digits-argument`
//!
//! Require `.toFixed()` to be called with an explicit digits argument.
//! `.toFixed()` defaults to `0` digits, but this should be explicit.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.toFixed()` calls without an explicit digits argument.
#[derive(Debug)]
pub struct RequireNumberToFixedDigitsArgument;

impl LintRule for RequireNumberToFixedDigitsArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-number-to-fixed-digits-argument".to_owned(),
            description: "Require `.toFixed()` to have an explicit digits argument".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let is_to_fixed = matches!(
            ctx.node(call.callee),
            Some(AstNode::StaticMemberExpression(member)) if member.property.as_str() == "toFixed"
        );

        if !is_to_fixed {
            return;
        }

        if !call.arguments.is_empty() {
            return;
        }

        let call_span = Span::new(call.span.start, call.span.end);
        // Fix: insert `0` inside the empty parens. The closing paren is at call.span.end - 1.
        let insert_span = Span::new(
            call.span.end.saturating_sub(1),
            call.span.end.saturating_sub(1),
        );
        ctx.report(Diagnostic {
            rule_name: "require-number-to-fixed-digits-argument".to_owned(),
            message: "`.toFixed()` should have an explicit digits argument".to_owned(),
            span: call_span,
            severity: Severity::Warning,
            help: Some("Add `0` as the digits argument".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Add `0` argument".to_owned(),
                edits: vec![Edit {
                    span: insert_span,
                    replacement: "0".to_owned(),
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

    starlint_rule_framework::lint_rule_test!(RequireNumberToFixedDigitsArgument);

    #[test]
    fn test_flags_to_fixed_no_args() {
        let diags = lint("const s = num.toFixed();");
        assert_eq!(diags.len(), 1, "should flag .toFixed() without args");
    }

    #[test]
    fn test_allows_to_fixed_with_arg() {
        let diags = lint("const s = num.toFixed(2);");
        assert!(diags.is_empty(), ".toFixed(2) should not be flagged");
    }

    #[test]
    fn test_allows_to_fixed_with_zero() {
        let diags = lint("const s = num.toFixed(0);");
        assert!(
            diags.is_empty(),
            ".toFixed(0) should not be flagged (explicit is fine)"
        );
    }
}
