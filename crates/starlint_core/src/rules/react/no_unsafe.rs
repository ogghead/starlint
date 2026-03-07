//! Rule: `react/no-unsafe`
//!
//! Warn when using unsafe lifecycle methods: `UNSAFE_componentWillMount`,
//! `UNSAFE_componentWillReceiveProps`, `UNSAFE_componentWillUpdate`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags usage of `UNSAFE_` lifecycle methods.
#[derive(Debug)]
pub struct NoUnsafe;

impl LintRule for NoUnsafe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-unsafe".to_owned(),
            description: "Disallow usage of unsafe lifecycle methods".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        let Some(key_node) = ctx.node(method.key) else {
            return;
        };
        let key_span = key_node.span();
        let source = ctx.source_text();
        let start = usize::try_from(key_span.start).unwrap_or(0);
        let end = usize::try_from(key_span.end).unwrap_or(0);
        let method_name = source.get(start..end).unwrap_or("");
        let id_span = key_span;

        let safe_name = match method_name {
            "UNSAFE_componentWillMount" => Some("componentDidMount"),
            "UNSAFE_componentWillReceiveProps" => Some("componentDidUpdate"),
            "UNSAFE_componentWillUpdate" => Some("getSnapshotBeforeUpdate"),
            _ => None,
        };

        if let Some(replacement) = safe_name {
            ctx.report(Diagnostic {
                rule_name: "react/no-unsafe".to_owned(),
                message: format!(
                    "`{method_name}` is unsafe and deprecated — use safe lifecycle methods instead"
                ),
                span: Span::new(method.span.start, method.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Rename to `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(id_span.start, id_span.end),
                        replacement: replacement.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnsafe)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_unsafe_component_will_mount() {
        let diags = lint("class Foo extends React.Component { UNSAFE_componentWillMount() {} }");
        assert_eq!(diags.len(), 1, "should flag UNSAFE_componentWillMount");
    }

    #[test]
    fn test_flags_unsafe_component_will_receive_props() {
        let diags =
            lint("class Foo extends React.Component { UNSAFE_componentWillReceiveProps() {} }");
        assert_eq!(
            diags.len(),
            1,
            "should flag UNSAFE_componentWillReceiveProps"
        );
    }

    #[test]
    fn test_allows_safe_lifecycle_methods() {
        let diags = lint(
            "class Foo extends React.Component { componentDidMount() {} render() { return null; } }",
        );
        assert!(
            diags.is_empty(),
            "safe lifecycle methods should not be flagged"
        );
    }
}
