//! Rule: `prefer-dom-node-dataset`
//!
//! Prefer `element.dataset.foo` over `element.getAttribute('data-foo')` or
//! `element.setAttribute('data-foo', value)`. The `dataset` API is more
//! readable and less error-prone.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `getAttribute`/`setAttribute` calls with `data-` prefixed string arguments.
#[derive(Debug)]
pub struct PreferDomNodeDataset;

/// Method names that operate on `data-*` attributes.
const DATA_ATTR_METHODS: &[&str] = &["getAttribute", "setAttribute"];

impl LintRule for PreferDomNodeDataset {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-dataset".to_owned(),
            description: "Prefer `element.dataset` over `getAttribute`/`setAttribute` with `data-*` attributes".to_owned(),
            category: Category::Suggestion,
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

        // Callee must be a static member expression like `el.getAttribute(...)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();

        if !DATA_ATTR_METHODS.contains(&method_name) {
            return;
        }

        // The first argument must be a string literal starting with "data-"
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(AstNode::StringLiteral(lit)) = ctx.node(*first_arg_id) else {
            return;
        };

        if !lit.value.starts_with("data-") {
            return;
        }

        let dataset_key = data_attr_to_camel_case(lit.value.as_str());
        let member_object = member.object;

        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let obj_span = ctx.node(member_object).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let obj_text = source
                .get(obj_span.start as usize..obj_span.end as usize)
                .unwrap_or("");
            if method_name == "getAttribute" {
                let replacement = format!("{obj_text}.dataset.{dataset_key}");
                Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                })
            } else if method_name == "setAttribute" && call.arguments.len() == 2 {
                call.arguments.get(1).and_then(|val_arg_id| {
                    let val_span = ctx.node(*val_arg_id).map_or(
                        starlint_ast::types::Span::EMPTY,
                        starlint_ast::AstNode::span,
                    );
                    let val_text = source.get(val_span.start as usize..val_span.end as usize)?;
                    let replacement = format!("{obj_text}.dataset.{dataset_key} = {val_text}");
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                })
            } else {
                None
            }
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-dom-node-dataset".to_owned(),
            message: format!(
                "Prefer `element.dataset` over `{method_name}` with a `data-*` attribute"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

/// Convert a `data-*` attribute name to its camelCase dataset key.
/// e.g. `data-foo-bar` -> `fooBar`
fn data_attr_to_camel_case(attr: &str) -> String {
    let without_prefix = attr.strip_prefix("data-").unwrap_or(attr);
    let mut result = String::new();
    let mut capitalize_next = false;
    for ch in without_prefix.chars() {
        if ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferDomNodeDataset)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_get_attribute_data() {
        let diags = lint("el.getAttribute('data-id');");
        assert_eq!(
            diags.len(),
            1,
            "getAttribute with data- attribute should be flagged"
        );
    }

    #[test]
    fn test_flags_set_attribute_data() {
        let diags = lint("el.setAttribute('data-name', 'foo');");
        assert_eq!(
            diags.len(),
            1,
            "setAttribute with data- attribute should be flagged"
        );
    }

    #[test]
    fn test_allows_get_attribute_non_data() {
        let diags = lint("el.getAttribute('class');");
        assert!(
            diags.is_empty(),
            "getAttribute without data- prefix should not be flagged"
        );
    }

    #[test]
    fn test_allows_dataset_access() {
        let diags = lint("el.dataset.id;");
        assert!(diags.is_empty(), "dataset access should not be flagged");
    }

    #[test]
    fn test_allows_set_attribute_non_data() {
        let diags = lint("el.setAttribute('id', 'main');");
        assert!(
            diags.is_empty(),
            "setAttribute without data- prefix should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_call() {
        let diags = lint("el.removeAttribute('data-id');");
        assert!(diags.is_empty(), "removeAttribute should not be flagged");
    }
}
