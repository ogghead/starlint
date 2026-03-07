//! Rule: `react/no-namespace`
//!
//! Error when JSX elements use namespaced names like `<ns:Component>`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

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
        Some(&[AstNodeType::JSXNamespacedName])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXNamespacedName(ns_name) = node else {
            return;
        };

        let namespace = ns_name.namespace.as_str();
        let name = ns_name.name.as_str();

        ctx.report(Diagnostic {
            rule_name: "react/no-namespace".to_owned(),
            message: format!("React does not support JSX namespaces — found `{namespace}:{name}`"),
            span: Span::new(ns_name.span.start, ns_name.span.end),
            severity: Severity::Error,
            help: None,
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Remove namespace prefix `{namespace}:`"),
                edits: vec![Edit {
                    span: Span::new(ns_name.span.start, ns_name.span.end),
                    replacement: name.to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNamespace)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_namespaced_element() {
        let diags = lint(r"const x = <ns:Component />;");
        // ns:Component generates a JSXNamespacedName node
        assert!(!diags.is_empty(), "should flag namespaced JSX element");
    }

    #[test]
    fn test_flags_namespaced_attribute() {
        let diags = lint(r#"const x = <div xml:lang="en" />;"#);
        // xml:lang generates a JSXNamespacedName for the attribute name
        assert!(!diags.is_empty(), "should flag namespaced JSX attribute");
    }

    #[test]
    fn test_allows_normal_element() {
        let diags = lint(r"const x = <Component />;");
        assert!(diags.is_empty(), "normal elements should not be flagged");
    }
}
