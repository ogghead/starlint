//! Rule: `prefer-string-slice` (unicorn)
//!
//! Prefer `String#slice()` over `String#substr()` and `String#substring()`.
//! `slice()` is more consistent and handles negative indices intuitively,
//! while `substr()` is deprecated and `substring()` swaps arguments silently
//! when the first is greater than the second.

#![allow(clippy::or_fun_call)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `substr()` and `substring()` usage.
#[derive(Debug)]
pub struct PreferStringSlice;

impl LintRule for PreferStringSlice {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-slice".to_owned(),
            description: "Prefer `String#slice()` over `substr()` and `substring()`".to_owned(),
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method = member.property.as_str();
        match method {
            "substr" | "substring" => {
                let source = ctx.source_text();
                let call_text = source
                    .get(call.span.start as usize..call.span.end as usize)
                    .unwrap_or("");
                let prop_span = call_text.find(method).map_or(
                    Span::new(call.span.start, call.span.end),
                    |offset| {
                        let start = call.span.start.saturating_add(offset as u32);
                        Span::new(start, start.saturating_add(method.len() as u32))
                    },
                );

                ctx.report(Diagnostic {
                    rule_name: "prefer-string-slice".to_owned(),
                    message: format!("Prefer `.slice()` over `.{method}()`"),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace `.{method}()` with `.slice()`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace `.{method}()` with `.slice()`"),
                        edits: vec![Edit {
                            span: prop_span,
                            replacement: "slice".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferStringSlice);

    #[test]
    fn test_flags_substr() {
        let diags = lint("str.substr(1, 3);");
        assert_eq!(diags.len(), 1, "substr should be flagged");
    }

    #[test]
    fn test_flags_substring() {
        let diags = lint("str.substring(1, 3);");
        assert_eq!(diags.len(), 1, "substring should be flagged");
    }

    #[test]
    fn test_allows_slice() {
        let diags = lint("str.slice(1, 3);");
        assert!(diags.is_empty(), "slice should not be flagged");
    }

    #[test]
    fn test_allows_other_methods() {
        let diags = lint("str.indexOf('x');");
        assert!(diags.is_empty(), "other methods should not be flagged");
    }
}
