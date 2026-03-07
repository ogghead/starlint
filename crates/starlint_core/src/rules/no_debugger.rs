//! Rule: `no-debugger`
//!
//! Disallow `debugger` statements. These should never appear in production code.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `debugger` statements and offers a safe fix to remove them.
#[derive(Debug)]
pub struct NoDebugger;

impl LintRule for NoDebugger {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-debugger".to_owned(),
            description: "Disallow `debugger` statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::DebuggerStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if let AstNode::DebuggerStatement(stmt) = node {
            let span = Span::new(stmt.span.start, stmt.span.end);
            ctx.report(Diagnostic {
                rule_name: "no-debugger".to_owned(),
                message: "Unexpected `debugger` statement".to_owned(),
                span,
                severity: Severity::Error,
                help: Some("Remove the `debugger` statement before deploying".to_owned()),
                fix: FixBuilder::new("Remove `debugger` statement", FixKind::SafeFix)
                    .delete(span)
                    .build(),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    #[test]
    fn test_flags_debugger_statement() {
        let source = "debugger;\nconst x = 1;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDebugger)];
        let diags = lint_source(source, "test.js", &rules);
        assert_eq!(diags.len(), 1, "should find one debugger statement");
        let first = diags.first();
        assert_eq!(
            first.map(|d| d.rule_name.as_str()),
            Some("no-debugger"),
            "rule name should match"
        );
        assert!(
            first.is_some_and(|d| d.fix.is_some()),
            "should provide a fix"
        );
    }

    #[test]
    fn test_clean_file_no_diagnostics() {
        let source = "const x = 1;\nexport default x;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDebugger)];
        let diags = lint_source(source, "test.js", &rules);
        assert!(diags.is_empty(), "clean file should have no diagnostics");
    }

    #[test]
    fn test_multiple_debugger_statements() {
        let source = "debugger;\nconst x = 1;\ndebugger;";
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDebugger)];
        let diags = lint_source(source, "test.js", &rules);
        assert_eq!(diags.len(), 2, "should find two debugger statements");
    }
}
