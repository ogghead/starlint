//! Rule: `no-eval`
//!
//! Disallow the use of `eval()`. `eval()` is dangerous because it executes
//! arbitrary code with the caller's privileges and can be exploited for
//! code injection attacks.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags calls to `eval()`.
#[derive(Debug)]
pub struct NoEval;

impl LintRule for NoEval {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-eval".to_owned(),
            description: "Disallow the use of `eval()`".to_owned(),
            category: Category::Style,
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

        let is_eval = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str() == "eval",
            Some(AstNode::StaticMemberExpression(member)) => {
                member.property.as_str() == "eval"
                    && matches!(
                        ctx.node(member.object),
                        Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "window"
                            || id.name.as_str() == "globalThis"
                    )
            }
            _ => false,
        };

        if is_eval {
            ctx.report(Diagnostic {
                rule_name: "no-eval".to_owned(),
                message: "`eval()` is a security risk and can be harmful".to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEval)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_eval() {
        let diags = lint("eval('code');");
        assert_eq!(diags.len(), 1, "eval() should be flagged");
    }

    #[test]
    fn test_flags_window_eval() {
        let diags = lint("window.eval('code');");
        assert_eq!(diags.len(), 1, "window.eval() should be flagged");
    }

    #[test]
    fn test_allows_non_eval() {
        let diags = lint("foo('code');");
        assert!(diags.is_empty(), "non-eval call should not be flagged");
    }

    #[test]
    fn test_allows_eval_as_property() {
        let diags = lint("obj.eval('code');");
        assert!(
            diags.is_empty(),
            "eval as property of non-global should not be flagged"
        );
    }
}
