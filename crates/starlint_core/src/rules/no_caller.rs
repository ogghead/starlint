//! Rule: `no-caller`
//!
//! Disallow use of `arguments.caller` and `arguments.callee`. These are
//! deprecated and forbidden in strict mode.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `arguments.caller` and `arguments.callee`.
#[derive(Debug)]
pub struct NoCaller;

impl LintRule for NoCaller {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-caller".to_owned(),
            description: "Disallow use of `arguments.caller` and `arguments.callee`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        let prop = member.property.as_str();
        if prop != "caller" && prop != "callee" {
            return;
        }

        let Some(AstNode::IdentifierReference(id)) = ctx.node(member.object) else {
            return;
        };

        if id.name.as_str() == "arguments" {
            ctx.report(Diagnostic {
                rule_name: "no-caller".to_owned(),
                message: format!("Avoid using `arguments.{prop}` — it is deprecated"),
                span: Span::new(member.span.start, member.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCaller)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_arguments_callee() {
        let diags = lint("function f() { return arguments.callee; }");
        assert_eq!(diags.len(), 1, "arguments.callee should be flagged");
    }

    #[test]
    fn test_flags_arguments_caller() {
        let diags = lint("function f() { return arguments.caller; }");
        assert_eq!(diags.len(), 1, "arguments.caller should be flagged");
    }

    #[test]
    fn test_allows_other_properties() {
        let diags = lint("function f() { return arguments.length; }");
        assert!(diags.is_empty(), "arguments.length should not be flagged");
    }
}
