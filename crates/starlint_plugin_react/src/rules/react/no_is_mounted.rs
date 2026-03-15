//! Rule: `react/no-is-mounted`
//!
//! Disallow usage of `isMounted()`. `isMounted` is an anti-pattern, is not
//! available when using ES6 classes, and is on its way to being officially
//! deprecated.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `this.isMounted()` calls.
#[derive(Debug)]
pub struct NoIsMounted;

impl LintRule for NoIsMounted {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-is-mounted".to_owned(),
            description: "Disallow usage of `isMounted()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let is_this_is_mounted = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                return;
            };
            member.property.as_str() == "isMounted"
                && matches!(ctx.node(member.object), Some(AstNode::ThisExpression(_)))
        };

        if is_this_is_mounted {
            ctx.report(Diagnostic {
                rule_name: "react/no-is-mounted".to_owned(),
                message: "`isMounted` is an anti-pattern — use a `_isMounted` instance variable or cancellable promises instead".to_owned(),
                span: Span::new(call.span.start, call.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoIsMounted);

    #[test]
    fn test_flags_is_mounted() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        if (this.isMounted()) {
            this.setState({ clicked: true });
        }
    }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "this.isMounted() should be flagged");
    }

    #[test]
    fn test_allows_other_method_calls() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ clicked: true });
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "other method calls should not be flagged");
    }
}
