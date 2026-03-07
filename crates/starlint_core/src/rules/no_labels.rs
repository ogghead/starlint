//! Rule: `no-labels`
//!
//! Disallow labeled statements. Labels are rarely needed and can make
//! code harder to understand.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags labeled statements.
#[derive(Debug)]
pub struct NoLabels;

impl LintRule for NoLabels {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-labels".to_owned(),
            description: "Disallow labeled statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::LabeledStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::LabeledStatement(stmt) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-labels".to_owned(),
            message: "Unexpected labeled statement".to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLabels)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_labeled_statement() {
        let diags = lint("outer: for (var i = 0; i < 10; i++) { break outer; }");
        assert_eq!(diags.len(), 1, "labeled statement should be flagged");
    }

    #[test]
    fn test_allows_unlabeled_loop() {
        let diags = lint("for (var i = 0; i < 10; i++) { break; }");
        assert!(diags.is_empty(), "unlabeled loop should not be flagged");
    }
}
