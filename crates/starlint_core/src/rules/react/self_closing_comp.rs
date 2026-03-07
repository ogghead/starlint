//! Rule: `react/self-closing-comp`
//!
//! Components without children should be self-closing. Writing `<Foo></Foo>`
//! when there are no children is unnecessarily verbose.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags JSX elements without children that are not self-closing.
#[derive(Debug)]
pub struct SelfClosingComp;

impl LintRule for SelfClosingComp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/self-closing-comp".to_owned(),
            description: "Components without children should be self-closing".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        // JSXElementNode has no closing_element field. Check if the opening element
        // is self-closing. If it's NOT self-closing and has no children, flag it.
        let Some(AstNode::JSXOpeningElement(opening)) = ctx.node(element.opening_element) else {
            return;
        };

        if opening.self_closing {
            return; // Already self-closing
        }

        if !element.children.is_empty() {
            return; // Has children, not applicable
        }

        let element_span = Span::new(element.span.start, element.span.end);
        let source = ctx.source_text();
        let open_start = usize::try_from(opening.span.start).unwrap_or(0);
        let open_end = usize::try_from(opening.span.end).unwrap_or(0);
        let opening_text = source.get(open_start..open_end).unwrap_or("");

        // Build self-closing tag: strip trailing ">" and append " />"
        let replacement = opening_text.strip_suffix('>').map_or_else(
            || format!("{opening_text} />"),
            |without_angle| {
                let t = without_angle.trim_end();
                format!("{t} />")
            },
        );

        ctx.report(Diagnostic {
            rule_name: "react/self-closing-comp".to_owned(),
            message: "Empty components should be self-closing".to_owned(),
            span: element_span,
            severity: Severity::Warning,
            help: Some("Use a self-closing tag instead".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Convert to self-closing tag".to_owned(),
                edits: vec![Edit {
                    span: element_span,
                    replacement,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(SelfClosingComp)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_element_with_closing_tag() {
        let source = "var x = <div></div>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "empty element with closing tag should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_component_with_closing_tag() {
        let source = "var x = <MyComponent></MyComponent>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "empty component with closing tag should be flagged"
        );
    }

    #[test]
    fn test_allows_self_closing() {
        let source = "var x = <div />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "self-closing element should not be flagged"
        );
    }

    #[test]
    fn test_allows_element_with_children() {
        let source = "var x = <div>hello</div>;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "element with children should not be flagged"
        );
    }
}
