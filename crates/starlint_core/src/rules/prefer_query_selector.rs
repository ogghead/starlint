//! Rule: `prefer-query-selector`
//!
//! Prefer `querySelector` / `querySelectorAll` over `getElementById`,
//! `getElementsByClassName`, `getElementsByTagName`, and
//! `getElementsByTagNameNS`. The `querySelector` family uses CSS selectors
//! and provides a more consistent, flexible API.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags legacy DOM query methods in favor of `querySelector` / `querySelectorAll`.
#[derive(Debug)]
pub struct PreferQuerySelector;

/// Legacy DOM query methods that should be replaced.
const LEGACY_METHODS: &[&str] = &[
    "getElementById",
    "getElementsByClassName",
    "getElementsByTagName",
    "getElementsByTagNameNS",
];

impl LintRule for PreferQuerySelector {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-query-selector".to_owned(),
            description:
                "Prefer `querySelector` / `querySelectorAll` over legacy DOM query methods"
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();

        if !LEGACY_METHODS.contains(&method_name) {
            return;
        }

        let suggestion = suggested_replacement(method_name);

        // Build fix for getElementById/getElementsByClassName/getElementsByTagName
        #[allow(clippy::as_conversions)]
        let fix = (call.arguments.len() == 1)
            .then(|| {
                call.arguments.first().and_then(|arg| {
                    let AstNode::StringLiteral(lit) = ctx.node(*arg)? else {
                        return None;
                    };
                    let source = ctx.source_text();
                    let obj_span = ctx.node(member.object)?.span();
                    let obj_text = source.get(obj_span.start as usize..obj_span.end as usize)?;
                    let (method, selector) = match method_name {
                        "getElementById" => ("querySelector", format!("#{}", lit.value)),
                        "getElementsByClassName" => ("querySelectorAll", format!(".{}", lit.value)),
                        "getElementsByTagName" => ("querySelectorAll", lit.value.clone()),
                        _ => return None,
                    };
                    let replacement = format!("{obj_text}.{method}('{selector}')");
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{method}('{selector}')`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                })
            })
            .flatten();

        ctx.report(Diagnostic {
            rule_name: "prefer-query-selector".to_owned(),
            message: format!("Prefer `{suggestion}` over `{method_name}`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!("Use `{suggestion}` with a CSS selector instead")),
            fix,
            labels: vec![],
        });
    }
}

/// Return the modern replacement for a legacy DOM query method.
fn suggested_replacement(method: &str) -> &'static str {
    if method == "getElementById" {
        "querySelector"
    } else {
        "querySelectorAll"
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferQuerySelector)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_get_element_by_id() {
        let diags = lint("document.getElementById('foo');");
        assert_eq!(diags.len(), 1, "getElementById should be flagged");
    }

    #[test]
    fn test_flags_get_elements_by_class_name() {
        let diags = lint("document.getElementsByClassName('bar');");
        assert_eq!(diags.len(), 1, "getElementsByClassName should be flagged");
    }

    #[test]
    fn test_flags_get_elements_by_tag_name() {
        let diags = lint("el.getElementsByTagName('div');");
        assert_eq!(diags.len(), 1, "getElementsByTagName should be flagged");
    }

    #[test]
    fn test_flags_get_elements_by_tag_name_ns() {
        let diags = lint("document.getElementsByTagNameNS('ns', 'div');");
        assert_eq!(diags.len(), 1, "getElementsByTagNameNS should be flagged");
    }

    #[test]
    fn test_allows_query_selector() {
        let diags = lint("document.querySelector('#foo');");
        assert!(diags.is_empty(), "querySelector should not be flagged");
    }

    #[test]
    fn test_allows_query_selector_all() {
        let diags = lint("document.querySelectorAll('.bar');");
        assert!(diags.is_empty(), "querySelectorAll should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("document.createElement('div');");
        assert!(
            diags.is_empty(),
            "unrelated DOM methods should not be flagged"
        );
    }
}
