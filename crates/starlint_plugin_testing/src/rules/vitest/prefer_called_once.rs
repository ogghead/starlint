//! Rule: `vitest/prefer-called-once`
//!
//! Suggest `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`.
//! The `toHaveBeenCalledOnce()` matcher is more readable and expressive
//! when asserting exactly one call.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-called-once";

/// Suggest `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`.
#[derive(Debug)]
pub struct PreferCalledOnce;

impl LintRule for PreferCalledOnce {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("toHaveBeenCalledTimes(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `.toHaveBeenCalledTimes(1)`.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "toHaveBeenCalledTimes" {
            return;
        }

        // Check that the single argument is the numeric literal `1`.
        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_one = match ctx.node(*first_arg) {
            Some(AstNode::NumericLiteral(lit)) => {
                #[allow(clippy::float_cmp)]
                {
                    lit.value == 1.0
                }
            }
            _ => false,
        };

        if is_one {
            // Build fix: replace `.toHaveBeenCalledTimes(1)` with `.toHaveBeenCalledOnce()`
            let source = ctx.source_text();
            let obj_span = ctx.node(member.object).map_or(Span::new(0, 0), |n| {
                let s = n.span();
                Span::new(s.start, s.end)
            });
            let obj_text = source
                .get(obj_span.start as usize..obj_span.end as usize)
                .unwrap_or("")
                .to_owned();
            let replacement = format!("{obj_text}.toHaveBeenCalledOnce()");
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `toHaveBeenCalledOnce()`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement,
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Prefer `toHaveBeenCalledOnce()` over `toHaveBeenCalledTimes(1)`"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `toHaveBeenCalledOnce()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferCalledOnce);

    #[test]
    fn test_flags_called_times_one() {
        let source = "expect(mock).toHaveBeenCalledTimes(1);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`toHaveBeenCalledTimes(1)` should be flagged"
        );
    }

    #[test]
    fn test_allows_called_times_other() {
        let source = "expect(mock).toHaveBeenCalledTimes(3);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledTimes(3)` should not be flagged"
        );
    }

    #[test]
    fn test_allows_called_once() {
        let source = "expect(mock).toHaveBeenCalledOnce();";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toHaveBeenCalledOnce()` should not be flagged"
        );
    }
}
