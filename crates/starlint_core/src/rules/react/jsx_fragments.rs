//! Rule: `react/jsx-fragments`
//!
//! Suggest using `<>` short syntax instead of `<React.Fragment>` when no key
//! prop is present.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-fragments";

/// Suggests using `<>` shorthand instead of `<React.Fragment>` when no `key`
/// prop is present.
#[derive(Debug)]
pub struct JsxFragments;

/// Check whether a JSX opening element has a `key` attribute.
fn has_key_prop(ctx: &LintContext<'_>, attributes: &[NodeId]) -> bool {
    attributes.iter().any(|attr_id| {
        let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) else {
            return false;
        };
        attr.name.as_str() == "key"
    })
}

/// Check if the element name is `React.Fragment`.
/// In `starlint_ast`, JSXOpeningElementNode.name is a String.
/// For member expressions like `React.Fragment`, the name is stored as "React.Fragment".
fn is_react_fragment(name: &str) -> bool {
    name == "React.Fragment"
}

impl LintRule for JsxFragments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `<>` shorthand over `<React.Fragment>` when no key is needed"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    #[allow(clippy::as_conversions)]
    #[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        if !is_react_fragment(&opening.name) {
            return;
        }

        // If there's a `key` prop, React.Fragment is required
        if has_key_prop(ctx, &opening.attributes) {
            return;
        }

        let opening_span = Span::new(opening.span.start, opening.span.end);

        // Build edits: replace opening tag with `<>`
        // Since starlint_ast doesn't have a closing_element field on JSXElementNode,
        // we replace just the opening tag and rely on the source text to find the closing tag.
        let source = ctx.source_text();
        let element_text = source
            .get(element.span.start as usize..element.span.end as usize)
            .unwrap_or("");

        // Find the closing tag `</React.Fragment>` in the element text
        let mut edits = vec![Edit {
            span: opening_span,
            replacement: "<>".to_owned(),
        }];

        if let Some(closing_start) = element_text.rfind("</React.Fragment>") {
            let closing_abs_start = element.span.start + closing_start as u32;
            let closing_abs_end = closing_abs_start + "</React.Fragment>".len() as u32;
            edits.push(Edit {
                span: Span::new(closing_abs_start, closing_abs_end),
                replacement: "</>".to_owned(),
            });
        }

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Prefer `<>` shorthand over `<React.Fragment>` when no `key` prop is needed"
                .to_owned(),
            span: Span::new(element.span.start, element.span.end),
            severity: Severity::Warning,
            help: Some("Replace `<React.Fragment>` with `<>`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with shorthand fragment syntax".to_owned(),
                edits,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxFragments)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_react_fragment_without_key() {
        let diags = lint("const el = <React.Fragment><div /></React.Fragment>;");
        assert_eq!(
            diags.len(),
            1,
            "should flag React.Fragment without key prop"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_react_fragment_with_key() {
        let diags = lint("const el = <React.Fragment key=\"k\"><div /></React.Fragment>;");
        assert!(
            diags.is_empty(),
            "should not flag React.Fragment with key prop"
        );
    }

    #[test]
    fn test_allows_short_syntax() {
        let diags = lint("const el = <><div /></>;");
        assert!(
            diags.is_empty(),
            "should not flag shorthand fragment syntax"
        );
    }
}
