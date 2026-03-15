//! Rule: `react/no-namespace`
//!
//! Error when JSX elements use namespaced names like `<ns:Component>`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags JSX namespaced element names.
#[derive(Debug)]
pub struct NoNamespace;

impl LintRule for NoNamespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-namespace".to_owned(),
            description: "Disallow namespaced JSX elements (e.g. `<ns:Component>`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement, AstNodeType::JSXAttribute])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::JSXOpeningElement(opening) => {
                if let Some((namespace, name)) = opening.name.split_once(':') {
                    report_namespaced(namespace, name, opening.span, ctx);
                }
            }
            AstNode::JSXAttribute(attr) => {
                if let Some((namespace, name)) = attr.name.split_once(':') {
                    report_namespaced(namespace, name, attr.span, ctx);
                }
            }
            _ => {}
        }
    }
}

/// Report a namespaced JSX name.
fn report_namespaced(
    namespace: &str,
    name: &str,
    span: starlint_ast::types::Span,
    ctx: &mut LintContext<'_>,
) {
    ctx.report(Diagnostic {
        rule_name: "react/no-namespace".to_owned(),
        message: format!("React does not support JSX namespaces — found `{namespace}:{name}`"),
        span: Span::new(span.start, span.end),
        severity: Severity::Error,
        help: None,
        fix: Some(Fix {
            kind: FixKind::SuggestionFix,
            message: format!("Remove namespace prefix `{namespace}:`"),
            edits: vec![Edit {
                span: Span::new(span.start, span.end),
                replacement: name.to_owned(),
            }],
            is_snippet: false,
        }),
        labels: vec![],
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNamespace);

    #[test]
    fn test_flags_namespaced_element() {
        let diags = lint(r"const x = <ns:Component />;");
        // ns:Component generates a JSXOpeningElement with name "ns:Component"
        assert!(!diags.is_empty(), "should flag namespaced JSX element");
    }

    #[test]
    fn test_flags_namespaced_attribute() {
        let diags = lint(r#"const x = <div xml:lang="en" />;"#);
        // xml:lang generates a JSXAttribute with name "xml:lang"
        assert!(!diags.is_empty(), "should flag namespaced JSX attribute");
    }

    #[test]
    fn test_allows_normal_element() {
        let diags = lint(r"const x = <Component />;");
        assert!(diags.is_empty(), "normal elements should not be flagged");
    }
}
