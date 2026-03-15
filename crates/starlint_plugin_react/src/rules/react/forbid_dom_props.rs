//! Rule: `react/forbid-dom-props`
//!
//! Warn when certain DOM props are used. Default: flags `id` prop on DOM elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags use of forbidden DOM props. By default, flags the `id` prop on
/// lowercase (DOM) elements as a hint that IDs are often an anti-pattern
/// in component-based architectures.
#[derive(Debug)]
pub struct ForbidDomProps;

/// Default set of forbidden DOM props.
const FORBIDDEN_PROPS: &[&str] = &["id"];

impl LintRule for ForbidDomProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/forbid-dom-props".to_owned(),
            description: "Warn when certain DOM props are used".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXAttribute(attr) = node else {
            return;
        };

        // Get the attribute name — in starlint_ast, attr.name is a String
        let attr_name = attr.name.as_str();
        // Skip namespaced names (e.g. "xml:lang")
        if attr_name.contains(':') {
            return;
        }

        // Only flag forbidden props
        if !FORBIDDEN_PROPS.contains(&attr_name) {
            return;
        }

        // Check if this attribute is on a DOM element by scanning the source
        // text backwards from the attribute to find the opening tag name.
        // We use the heuristic that the JSXAttribute's parent is a JSXOpeningElement
        // whose name starts with a lowercase letter.
        let source = ctx.source_text();
        let attr_start = usize::try_from(attr.span.start).unwrap_or(0);
        if attr_start == 0 {
            return;
        }

        // Scan backward to find `<tagname` pattern
        let before = &source[..attr_start];
        // Find the last `<` before this attribute
        if let Some(lt_pos) = before.rfind('<') {
            let after_lt = &source[lt_pos.saturating_add(1)..attr_start];
            // Extract the tag name (first word after `<`)
            let tag_name = after_lt.split_whitespace().next().unwrap_or("");
            if !tag_name.is_empty()
                && tag_name
                    .as_bytes()
                    .first()
                    .is_some_and(|&b| b.is_ascii_lowercase())
            {
                let attr_span = Span::new(attr.span.start, attr.span.end);
                let fix =
                    FixBuilder::new(format!("Remove `{attr_name}` prop"), FixKind::SuggestionFix)
                        .edit(fix_utils::remove_jsx_attr(source, attr_span))
                        .build();
                ctx.report(Diagnostic {
                    rule_name: "react/forbid-dom-props".to_owned(),
                    message: format!("Prop `{attr_name}` is forbidden on DOM elements"),
                    span: attr_span,
                    severity: Severity::Warning,
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

    starlint_rule_framework::lint_rule_test!(ForbidDomProps);

    #[test]
    fn test_flags_id_on_dom_element() {
        let source = r#"const x = <div id="main" />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "id prop on DOM element should be flagged");
    }

    #[test]
    fn test_allows_id_on_component() {
        let source = r#"const x = <MyComponent id="main" />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "id prop on React component should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_props_on_dom_element() {
        let source = r#"const x = <div className="foo" />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-forbidden prop on DOM element should not be flagged"
        );
    }
}
