//! Rule: `react/no-array-index-key`
//!
//! Warn when an array index is used as the `key` prop in a `.map()` call.
//! Using index as key can cause issues with component state when the list is
//! reordered.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags usage of array index as the `key` prop inside `.map()` callbacks.
#[derive(Debug)]
pub struct NoArrayIndexKey;

impl LintRule for NoArrayIndexKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-array-index-key".to_owned(),
            description: "Disallow usage of array index as key".to_owned(),
            category: Category::Correctness,
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

        // Check if this is a .map() call
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "map" {
            return;
        }

        // Get the callback argument
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        // Extract the index parameter name and body span from the callback
        let (param_name, body_start, body_end) = match ctx.node(*first_arg_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => {
                let Some(param_id) = arrow.params.get(1) else {
                    return;
                };
                let Some(name) = extract_param_name(*param_id, ctx) else {
                    return;
                };
                (name, arrow.span.start, arrow.span.end)
            }
            Some(AstNode::Function(func)) => {
                let Some(param_id) = func.params.get(1) else {
                    return;
                };
                let Some(name) = extract_param_name(*param_id, ctx) else {
                    return;
                };
                (name, func.span.start, func.span.end)
            }
            _ => return,
        };

        // Scan the callback body source text for `key={indexParam}` pattern
        let source = ctx.source_text();
        let start = usize::try_from(body_start).unwrap_or(0);
        let end = usize::try_from(body_end).unwrap_or(0);
        if start < source.len() && end <= source.len() && start < end {
            let body_source = &source[start..end];
            let key_pattern = format!("key={{{param_name}}}");
            if body_source.contains(&key_pattern) {
                ctx.report(Diagnostic {
                    rule_name: "react/no-array-index-key".to_owned(),
                    message: format!(
                        "Do not use array index `{param_name}` as `key` — use a stable identifier instead"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Extract the binding identifier name from a formal parameter node.
fn extract_param_name(param_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    match ctx.node(param_id) {
        Some(AstNode::BindingIdentifier(id)) => Some(id.name.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArrayIndexKey)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_index_as_key_in_map() {
        let diags = lint(r"const x = items.map((item, index) => <div key={index}>{item}</div>);");
        assert_eq!(diags.len(), 1, "should flag array index used as key");
    }

    #[test]
    fn test_allows_stable_key() {
        let diags = lint(r"const x = items.map((item) => <div key={item.id}>{item.name}</div>);");
        assert!(diags.is_empty(), "stable key should not be flagged");
    }

    #[test]
    fn test_allows_map_without_key() {
        let diags = lint(r"const x = items.map((item) => <div>{item.name}</div>);");
        assert!(
            diags.is_empty(),
            "map without key should not be flagged by this rule"
        );
    }
}
