//! Rule: `prefer-classlist-toggle`
//!
//! Prefer `classList.toggle(class, force)` over `classList.add()` or
//! `classList.remove()`. The `toggle` method with a second argument is
//! often cleaner, especially when conditionally adding or removing a class.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `classList.add()` and `classList.remove()` calls, suggesting `classList.toggle()`.
#[derive(Debug)]
pub struct PreferClasslistToggle;

/// Method names on `classList` that could be replaced by `toggle`.
const TOGGLEABLE_METHODS: &[&str] = &["add", "remove"];

impl LintRule for PreferClasslistToggle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-classlist-toggle".to_owned(),
            description: "Prefer `classList.toggle()` over `classList.add()`/`classList.remove()`"
                .to_owned(),
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

        // Callee must be a static member expression like `X.classList.add(...)`
        let Some(AstNode::StaticMemberExpression(outer)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = outer.property.as_str();

        if !TOGGLEABLE_METHODS.contains(&method_name) {
            return;
        }

        // The object of the outer member must be `?.classList`
        let Some(AstNode::StaticMemberExpression(inner)) = ctx.node(outer.object) else {
            return;
        };

        if inner.property.as_str() != "classList" {
            return;
        }

        // Fix: el.classList.add('x') -> el.classList.toggle('x', true)
        //      el.classList.remove('x') -> el.classList.toggle('x', false)
        #[allow(clippy::as_conversions)]
        let fix = (call.arguments.len() == 1)
            .then(|| {
                let source = ctx.source_text();
                let el_span = ctx.node(inner.object).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );
                let arg_span = call
                    .arguments
                    .first()
                    .and_then(|&id| ctx.node(id))
                    .map(starlint_ast::AstNode::span);
                match (
                    source.get(el_span.start as usize..el_span.end as usize),
                    arg_span.and_then(|s| source.get(s.start as usize..s.end as usize)),
                ) {
                    (Some(el_text), Some(arg_text)) => {
                        let force = if method_name == "add" {
                            "true"
                        } else {
                            "false"
                        };
                        let replacement =
                            format!("{el_text}.classList.toggle({arg_text}, {force})");
                        Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        })
                    }
                    _ => None,
                }
            })
            .flatten();

        ctx.report(Diagnostic {
            rule_name: "prefer-classlist-toggle".to_owned(),
            message: format!("Prefer `classList.toggle()` over `classList.{method_name}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferClasslistToggle);

    #[test]
    fn test_flags_classlist_add() {
        let diags = lint("el.classList.add('active');");
        assert_eq!(diags.len(), 1, "classList.add should be flagged");
    }

    #[test]
    fn test_flags_classlist_remove() {
        let diags = lint("el.classList.remove('active');");
        assert_eq!(diags.len(), 1, "classList.remove should be flagged");
    }

    #[test]
    fn test_allows_classlist_toggle() {
        let diags = lint("el.classList.toggle('active');");
        assert!(diags.is_empty(), "classList.toggle should not be flagged");
    }

    #[test]
    fn test_allows_classlist_contains() {
        let diags = lint("el.classList.contains('active');");
        assert!(diags.is_empty(), "classList.contains should not be flagged");
    }

    #[test]
    fn test_allows_non_classlist_add() {
        let diags = lint("set.add('value');");
        assert!(diags.is_empty(), "non-classList add should not be flagged");
    }

    #[test]
    fn test_allows_non_classlist_remove() {
        let diags = lint("map.remove('key');");
        assert!(
            diags.is_empty(),
            "non-classList remove should not be flagged"
        );
    }
}
