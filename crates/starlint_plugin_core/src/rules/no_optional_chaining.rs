//! Rule: `no-optional-chaining`
//!
//! Flag use of optional chaining (`?.`). Some codebases prefer explicit
//! null checks over optional chaining for clarity or compatibility.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags any optional chaining expression (`?.`).
#[derive(Debug)]
pub struct NoOptionalChaining;

impl LintRule for NoOptionalChaining {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-optional-chaining".to_owned(),
            description: "Disallow optional chaining (`?.`)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ChainExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ChainExpression(chain) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-optional-chaining".to_owned(),
            message: "Unexpected use of optional chaining (`?.`)".to_owned(),
            span: Span::new(chain.span.start, chain.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoOptionalChaining);

    #[test]
    fn test_flags_optional_member_access() {
        let diags = lint("foo?.bar;");
        assert_eq!(diags.len(), 1, "optional member access should be flagged");
    }

    #[test]
    fn test_flags_optional_call() {
        let diags = lint("foo?.();");
        assert_eq!(diags.len(), 1, "optional call should be flagged");
    }

    #[test]
    fn test_allows_regular_member_access() {
        let diags = lint("foo.bar;");
        assert!(
            diags.is_empty(),
            "regular member access should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_call() {
        let diags = lint("foo();");
        assert!(diags.is_empty(), "regular call should not be flagged");
    }

    #[test]
    fn test_flags_chained_optional() {
        let diags = lint("foo?.bar?.baz;");
        // A deeply chained `?.` expression is a single ChainExpression
        assert!(
            !diags.is_empty(),
            "chained optional access should be flagged"
        );
    }
}
