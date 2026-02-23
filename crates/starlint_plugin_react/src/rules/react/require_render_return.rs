//! Rule: `react/require-render-return`
//!
//! Require a return statement in `render()`. A `render` method that does not
//! return anything will cause the component to render `undefined`, which is
//! almost always a bug.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `render()` methods without a return statement.
#[derive(Debug)]
pub struct RequireRenderReturn;

impl LintRule for RequireRenderReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/require-render-return".to_owned(),
            description: "Require a return statement in `render()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        let method_name = ctx.node(method.key).and_then(|n| match n {
            AstNode::IdentifierReference(id) => Some(id.name.as_str()),
            AstNode::BindingIdentifier(id) => Some(id.name.as_str()),
            _ => None,
        });

        let Some("render") = method_name else {
            return;
        };

        // Resolve: method.value -> Function -> body -> FunctionBody
        let func_body = ctx
            .node(method.value)
            .and_then(|n| n.as_function())
            .and_then(|f| f.body)
            .and_then(|body_id| ctx.node(body_id))
            .and_then(|n| n.as_function_body());
        let Some(body) = func_body else {
            return;
        };

        // Check if the body contains at least one return statement at the top level
        let has_return = body
            .statements
            .iter()
            .any(|&stmt_id| matches!(ctx.node(stmt_id), Some(AstNode::ReturnStatement(_))));

        if !has_return {
            ctx.report(Diagnostic {
                rule_name: "react/require-render-return".to_owned(),
                message: "`render()` method must contain a return statement".to_owned(),
                span: Span::new(method.span.start, method.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireRenderReturn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_render_without_return() {
        let source = r"
class MyComponent extends React.Component {
    render() {
        console.log('no return');
    }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "render without return should be flagged");
    }

    #[test]
    fn test_allows_render_with_return() {
        let source = r"
class MyComponent extends React.Component {
    render() {
        return <div />;
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "render with return should not be flagged");
    }

    #[test]
    fn test_allows_render_returning_null() {
        let source = r"
class MyComponent extends React.Component {
    render() {
        return null;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "render returning null should not be flagged"
        );
    }
}
