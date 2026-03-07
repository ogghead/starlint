//! Rule: `prefer-code-point` (unicorn)
//!
//! Prefer `String#codePointAt()` over `String#charCodeAt()` and
//! `String.fromCodePoint()` over `String.fromCharCode()`.
//! Code points handle surrogate pairs correctly while char codes do not.

#![allow(clippy::or_fun_call)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `charCodeAt` and `fromCharCode` usage.
#[derive(Debug)]
pub struct PreferCodePoint;

impl LintRule for PreferCodePoint {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-code-point".to_owned(),
            description:
                "Prefer `codePointAt` over `charCodeAt` and `fromCodePoint` over `fromCharCode`"
                    .to_owned(),
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
        let replacement = match method {
            "charCodeAt" => "codePointAt",
            "fromCharCode" => "fromCodePoint",
            _ => return,
        };

        // Compute property span from source text
        let source = ctx.source_text();
        let call_text = source
            .get(call.span.start as usize..call.span.end as usize)
            .unwrap_or("");
        let prop_span =
            call_text
                .find(method)
                .map_or(Span::new(call.span.start, call.span.end), |offset| {
                    let start = call.span.start.saturating_add(offset as u32);
                    Span::new(start, start.saturating_add(method.len() as u32))
                });

        ctx.report(Diagnostic {
            rule_name: "prefer-code-point".to_owned(),
            message: format!("Prefer `{replacement}()` over `{method}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace `{method}` with `{replacement}`")),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace `{method}` with `{replacement}`"),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: replacement.to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferCodePoint)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_char_code_at() {
        let diags = lint("str.charCodeAt(0);");
        assert_eq!(diags.len(), 1, "charCodeAt should be flagged");
    }

    #[test]
    fn test_flags_from_char_code() {
        let diags = lint("String.fromCharCode(65);");
        assert_eq!(diags.len(), 1, "fromCharCode should be flagged");
    }

    #[test]
    fn test_allows_code_point_at() {
        let diags = lint("str.codePointAt(0);");
        assert!(diags.is_empty(), "codePointAt should not be flagged");
    }

    #[test]
    fn test_allows_from_code_point() {
        let diags = lint("String.fromCodePoint(65);");
        assert!(diags.is_empty(), "fromCodePoint should not be flagged");
    }
}
