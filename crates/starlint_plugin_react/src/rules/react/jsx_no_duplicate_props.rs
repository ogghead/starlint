//! Rule: `react/jsx-no-duplicate-props`
//!
//! Error when a JSX element has duplicate prop names.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-duplicate-props";

/// Flags JSX elements that have duplicate attribute/prop names.
#[derive(Debug)]
pub struct JsxNoDuplicateProps;

impl LintRule for JsxNoDuplicateProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow duplicate props in JSX elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // Collect attribute names and spans first to avoid borrow conflicts.
        let mut attrs: Vec<(String, Span)> = Vec::new();
        for attr_id in &opening.attributes {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                attrs.push((attr.name.clone(), Span::new(attr.span.start, attr.span.end)));
            }
        }

        let mut seen: HashSet<String> = HashSet::new();
        for (name, attr_span) in &attrs {
            if !seen.insert(name.clone()) {
                let fix = FixBuilder::new(
                    format!("Remove duplicate `{name}` prop"),
                    FixKind::SuggestionFix,
                )
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), *attr_span))
                .build();
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("Duplicate prop `{name}` found on JSX element"),
                    span: *attr_span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(JsxNoDuplicateProps);

    #[test]
    fn test_flags_duplicate_props() {
        let diags = lint(r#"const el = <div className="a" className="b" />;"#);
        assert_eq!(diags.len(), 1, "should flag duplicate className");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_unique_props() {
        let diags = lint(r#"const el = <div className="a" id="b" />;"#);
        assert!(diags.is_empty(), "should not flag unique props");
    }

    #[test]
    fn test_flags_multiple_duplicates() {
        let diags = lint(r#"const el = <div id="a" id="b" id="c" />;"#);
        assert_eq!(diags.len(), 2, "should flag each duplicate occurrence");
    }
}
