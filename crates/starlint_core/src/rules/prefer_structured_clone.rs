//! Rule: `prefer-structured-clone` (unicorn)
//!
//! Prefer `structuredClone()` over `JSON.parse(JSON.stringify())` for
//! deep cloning objects. `structuredClone` is more efficient and handles
//! more data types correctly.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `JSON.parse(JSON.stringify(x))` patterns.
#[derive(Debug)]
pub struct PreferStructuredClone;

impl LintRule for PreferStructuredClone {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-structured-clone".to_owned(),
            description: "Prefer structuredClone over JSON.parse(JSON.stringify())".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for JSON.parse(...)
        if !is_json_method_call(call.callee, "parse", ctx) {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // The argument must be JSON.stringify(...)
        let Some(&arg_id) = call.arguments.first() else {
            return;
        };

        let is_json_stringify = match ctx.node(arg_id) {
            Some(AstNode::CallExpression(inner_call)) => {
                is_json_method_call(inner_call.callee, "stringify", ctx)
                    && inner_call.arguments.len() == 1
            }
            _ => false,
        };

        if is_json_stringify {
            // Extract the inner argument text for the fix
            let fix = if let Some(AstNode::CallExpression(inner_call)) = ctx.node(arg_id) {
                if let Some(&inner_arg_id) = inner_call.arguments.first() {
                    if let Some(inner_arg_node) = ctx.node(inner_arg_id) {
                        let inner_span = inner_arg_node.span();
                        let source = ctx.source_text();
                        let arg_text = source
                            .get(inner_span.start as usize..inner_span.end as usize)
                            .unwrap_or("")
                            .to_owned();
                        (!arg_text.is_empty()).then(|| Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `structuredClone({arg_text})`"),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement: format!("structuredClone({arg_text})"),
                            }],
                            is_snippet: false,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "prefer-structured-clone".to_owned(),
                message: "Prefer `structuredClone(x)` over `JSON.parse(JSON.stringify(x))`"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Use `structuredClone()` instead".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a callee `NodeId` is `JSON.methodName`.
fn is_json_method_call(callee_id: NodeId, method: &str, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(callee_id) else {
        return false;
    };

    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return false;
    };

    obj.name == "JSON" && member.property == method
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferStructuredClone)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_json_parse_stringify() {
        let diags = lint("var copy = JSON.parse(JSON.stringify(obj));");
        assert_eq!(
            diags.len(),
            1,
            "JSON.parse(JSON.stringify()) should be flagged"
        );
    }

    #[test]
    fn test_allows_structured_clone() {
        let diags = lint("var copy = structuredClone(obj);");
        assert!(diags.is_empty(), "structuredClone should not be flagged");
    }

    #[test]
    fn test_allows_json_parse_alone() {
        let diags = lint("var data = JSON.parse(text);");
        assert!(diags.is_empty(), "JSON.parse alone should not be flagged");
    }

    #[test]
    fn test_allows_json_stringify_alone() {
        let diags = lint("var text = JSON.stringify(obj);");
        assert!(
            diags.is_empty(),
            "JSON.stringify alone should not be flagged"
        );
    }
}
