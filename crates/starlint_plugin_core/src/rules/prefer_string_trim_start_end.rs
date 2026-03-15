//! Rule: `prefer-string-trim-start-end`
//!
//! Prefer `.trimStart()` / `.trimEnd()` over the deprecated
//! `.trimLeft()` / `.trimRight()` aliases.

#![allow(clippy::or_fun_call)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.trimLeft()` and `.trimRight()` — use `.trimStart()` / `.trimEnd()`.
#[derive(Debug)]
pub struct PreferStringTrimStartEnd;

impl LintRule for PreferStringTrimStartEnd {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-trim-start-end".to_owned(),
            description: "Prefer `.trimStart()` / `.trimEnd()` over `.trimLeft()` / `.trimRight()`"
                .to_owned(),
            category: Category::Style,
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
        let replacement_method = match method {
            "trimLeft" => "trimStart",
            "trimRight" => "trimEnd",
            _ => return,
        };

        // Only flag zero-argument calls.
        if !call.arguments.is_empty() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-string-trim-start-end".to_owned(),
            message: format!("Use `.{replacement_method}()` instead of `.{method}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Replace `.{method}()` with `.{replacement_method}()`"
            )),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `.{method}` with `.{replacement_method}`"),
                edits: vec![Edit {
                    span: {
                        let source = ctx.source_text();
                        let call_text = source
                            .get(call.span.start as usize..call.span.end as usize)
                            .unwrap_or("");
                        call_text.find(method).map_or(
                            Span::new(call.span.start, call.span.end),
                            |offset| {
                                let start = call.span.start.saturating_add(offset as u32);
                                Span::new(start, start.saturating_add(method.len() as u32))
                            },
                        )
                    },
                    replacement: replacement_method.to_owned(),
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

    starlint_rule_framework::lint_rule_test!(PreferStringTrimStartEnd);

    #[test]
    fn test_flags_trim_left() {
        let diags = lint("str.trimLeft();");
        assert_eq!(diags.len(), 1, "should flag .trimLeft()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("trimStart"),
            "fix should replace with trimStart"
        );
    }

    #[test]
    fn test_flags_trim_right() {
        let diags = lint("str.trimRight();");
        assert_eq!(diags.len(), 1, "should flag .trimRight()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("trimEnd"),
            "fix should replace with trimEnd"
        );
    }

    #[test]
    fn test_allows_trim_start() {
        let diags = lint("str.trimStart();");
        assert!(diags.is_empty(), ".trimStart() should not be flagged");
    }

    #[test]
    fn test_allows_trim_end() {
        let diags = lint("str.trimEnd();");
        assert!(diags.is_empty(), ".trimEnd() should not be flagged");
    }
}
