//! Rule: `react/no-render-return-value`
//!
//! Warn when the return value of `ReactDOM.render()` is used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/no-render-return-value";

/// Flags usage of the return value of `ReactDOM.render()`.
#[derive(Debug)]
pub struct NoRenderReturnValue;

/// Check if a call expression's callee is `ReactDOM.render(...)`.
fn is_react_dom_render(callee_id: NodeId, ctx: &LintContext<'_>) -> bool {
    if let Some(AstNode::StaticMemberExpression(member)) = ctx.node(callee_id) {
        if member.property.as_str() != "render" {
            return false;
        }
        if let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) {
            return obj.name.as_str() == "ReactDOM";
        }
    }
    false
}

impl LintRule for NoRenderReturnValue {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow using the return value of `ReactDOM.render()`".to_owned(),
            category: Category::Correctness,
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

        if !is_react_dom_render(call.callee, ctx) {
            return;
        }

        // Check the surrounding source to determine if the return value is used.
        let src = ctx.source_text();
        let start = usize::try_from(call.span.start).unwrap_or(0);
        let before = &src[..start];
        let trimmed = before.trim_end();

        // If the call is preceded by `=` (assignment or variable declaration),
        // the return value is being used.
        let return_value_used =
            trimmed.ends_with('=') || trimmed.ends_with('(') || trimmed.ends_with(',');

        if return_value_used {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use the return value of `ReactDOM.render()` — it is a legacy API"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
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

    starlint_rule_framework::lint_rule_test!(NoRenderReturnValue);

    #[test]
    fn test_flags_assigned_render_return_value() {
        let diags =
            lint("const instance = ReactDOM.render(<App />, document.getElementById('root'));");
        assert_eq!(
            diags.len(),
            1,
            "should flag using the return value of ReactDOM.render()"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_standalone_render_call() {
        let diags = lint("ReactDOM.render(<App />, document.getElementById('root'));");
        assert!(
            diags.is_empty(),
            "should not flag when return value is not used"
        );
    }

    #[test]
    fn test_flags_render_in_assignment() {
        let diags = lint("let x;\nx = ReactDOM.render(<App />, el);");
        assert_eq!(
            diags.len(),
            1,
            "should flag assignment of ReactDOM.render() return value"
        );
    }
}
