//! Rule: `no-ternary`
//!
//! Disallow ternary operators. Some teams prefer `if/else` statements
//! for readability.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags all ternary (conditional) expressions.
#[derive(Debug)]
pub struct NoTernary;

impl LintRule for NoTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-ternary".to_owned(),
            description: "Disallow ternary operators".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ConditionalExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ConditionalExpression(cond) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-ternary".to_owned(),
            message: "Unexpected use of ternary operator".to_owned(),
            span: Span::new(cond.span.start, cond.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoTernary)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_ternary() {
        let diags = lint("var x = a ? b : c;");
        assert_eq!(diags.len(), 1, "ternary expression should be flagged");
    }

    #[test]
    fn test_allows_if_else() {
        let diags = lint("var x; if (a) { x = b; } else { x = c; }");
        assert!(diags.is_empty(), "if-else should not be flagged");
    }
}
