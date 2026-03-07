//! Rule: `vitest/prefer-to-be-falsy`
//!
//! Suggest `toBeFalsy()` over `toBe(false)`. The `toBeFalsy()` matcher is
//! more idiomatic in Vitest for checking falsy values and provides clearer
//! intent.

#![allow(clippy::or_fun_call)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-to-be-falsy";

/// Suggest `toBeFalsy()` over `toBe(false)`.
#[derive(Debug)]
pub struct PreferToBeFalsy;

impl LintRule for PreferToBeFalsy {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toBeFalsy()` over `toBe(false)`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::map_unwrap_or
    )]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `.toBe(false)`.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "toBe" {
            return;
        }

        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_false =
            matches!(ctx.node(*first_arg), Some(AstNode::BooleanLiteral(lit)) if !lit.value);

        if is_false {
            // Replace from the property name start to end of call: `toBe(false)` -> `toBeFalsy()`
            // Compute property span from source text
            let source = ctx.source_text();
            let call_text = source
                .get(call.span.start as usize..call.span.end as usize)
                .unwrap_or("");
            let fix_span = call_text
                .find("toBe")
                .map_or(Span::new(call.span.start, call.span.end), |offset| {
                    Span::new(call.span.start.saturating_add(offset as u32), call.span.end)
                });
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Prefer `toBeFalsy()` over `toBe(false)`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace `toBe(false)` with `toBeFalsy()`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `toBeFalsy()`".to_owned(),
                    edits: vec![Edit {
                        span: fix_span,
                        replacement: "toBeFalsy()".to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferToBeFalsy)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_to_be_false() {
        let source = "expect(value).toBe(false);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`.toBe(false)` should be flagged");
    }

    #[test]
    fn test_allows_to_be_falsy() {
        let source = "expect(value).toBeFalsy();";
        let diags = lint(source);
        assert!(diags.is_empty(), "`.toBeFalsy()` should not be flagged");
    }

    #[test]
    fn test_allows_to_be_true() {
        let source = "expect(value).toBe(true);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`.toBe(true)` should not be flagged by this rule"
        );
    }
}
