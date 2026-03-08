//! Rule: `no-multi-assign`
//!
//! Disallow chained assignment expressions like `a = b = c = 5`.
//! Chained assignments are hard to read and can lead to unexpected behavior.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags chained assignment expressions.
#[derive(Debug)]
pub struct NoMultiAssign;

impl LintRule for NoMultiAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-multi-assign".to_owned(),
            description: "Disallow use of chained assignment expressions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Check if the right side is also an assignment
        if matches!(
            ctx.node(assign.right),
            Some(AstNode::AssignmentExpression(_))
        ) {
            ctx.report(Diagnostic {
                rule_name: "no-multi-assign".to_owned(),
                message: "Unexpected chained assignment".to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMultiAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_chained_assignment() {
        let diags = lint("a = b = c = 5;");
        assert!(!diags.is_empty(), "chained assignment should be flagged");
    }

    #[test]
    fn test_allows_single_assignment() {
        let diags = lint("a = 5;");
        assert!(diags.is_empty(), "single assignment should not be flagged");
    }
}
