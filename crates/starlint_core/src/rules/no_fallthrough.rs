//! Rule: `no-fallthrough`
//!
//! Disallow fallthrough of `case` statements in `switch`. Unintentional
//! fallthrough is a common source of bugs. Cases that intentionally fall
//! through should have a `// falls through` comment (not yet supported).
//!
//! Note: This is a basic implementation that does not yet check for
//! `// falls through` or `// no break` comments. A full implementation
//! requires comment extraction infrastructure.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags switch case fallthrough (cases without `break`, `return`, or `throw`).
#[derive(Debug)]
pub struct NoFallthrough;

impl LintRule for NoFallthrough {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-fallthrough".to_owned(),
            description: "Disallow fallthrough of `case` statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::SwitchStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::SwitchStatement(switch) = node else {
            return;
        };

        let cases = &switch.cases;
        let case_count = cases.len();

        // Collect fallthrough cases first to avoid borrow issues
        let mut fallthrough_spans: Vec<Span> = Vec::new();

        for (i, case_id) in cases.iter().enumerate() {
            // Skip the last case — no fallthrough possible
            let is_last = i.saturating_add(1) >= case_count;
            if is_last {
                continue;
            }

            let Some(AstNode::SwitchCase(case)) = ctx.node(*case_id) else {
                continue;
            };

            // Empty cases are intentional fallthrough (grouping)
            if case.consequent.is_empty() {
                continue;
            }

            // Check if the case ends with a terminator
            if !ends_with_terminator(&case.consequent, ctx) {
                fallthrough_spans.push(Span::new(case.span.start, case.span.end));
            }
        }

        for span in fallthrough_spans {
            ctx.report(Diagnostic {
                rule_name: "no-fallthrough".to_owned(),
                message: "Expected a `break` statement before falling through to the next case"
                    .to_owned(),
                span,
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a list of statements ends with a control flow terminator.
fn ends_with_terminator(stmts: &[NodeId], ctx: &LintContext<'_>) -> bool {
    let Some(last_id) = stmts.last() else {
        return false;
    };

    let Some(last) = ctx.node(*last_id) else {
        return false;
    };

    match last {
        AstNode::ReturnStatement(_)
        | AstNode::ThrowStatement(_)
        | AstNode::BreakStatement(_)
        | AstNode::ContinueStatement(_) => true,
        AstNode::BlockStatement(block) => ends_with_terminator(&block.body, ctx),
        AstNode::IfStatement(if_stmt) => {
            // Both branches must terminate
            let consequent_terminates = statement_terminates(if_stmt.consequent, ctx);
            let alternate_terminates = if_stmt
                .alternate
                .is_some_and(|alt_id| statement_terminates(alt_id, ctx));
            consequent_terminates && alternate_terminates
        }
        _ => false,
    }
}

/// Check if a single statement terminates control flow.
fn statement_terminates(stmt_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(stmt) = ctx.node(stmt_id) else {
        return false;
    };
    match stmt {
        AstNode::ReturnStatement(_)
        | AstNode::ThrowStatement(_)
        | AstNode::BreakStatement(_)
        | AstNode::ContinueStatement(_) => true,
        AstNode::BlockStatement(block) => ends_with_terminator(&block.body, ctx),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoFallthrough)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_fallthrough() {
        let diags = lint("switch(x) { case 1: foo(); case 2: bar(); break; }");
        assert_eq!(diags.len(), 1, "case without break should be flagged");
    }

    #[test]
    fn test_allows_break() {
        let diags = lint("switch(x) { case 1: foo(); break; case 2: bar(); break; }");
        assert!(diags.is_empty(), "cases with break should not be flagged");
    }

    #[test]
    fn test_allows_return() {
        let diags = lint("function f(x) { switch(x) { case 1: return 1; case 2: return 2; } }");
        assert!(diags.is_empty(), "cases with return should not be flagged");
    }

    #[test]
    fn test_allows_throw() {
        let diags = lint("switch(x) { case 1: throw new Error(); case 2: break; }");
        assert!(diags.is_empty(), "cases with throw should not be flagged");
    }

    #[test]
    fn test_allows_empty_case_grouping() {
        let diags = lint("switch(x) { case 1: case 2: foo(); break; }");
        assert!(
            diags.is_empty(),
            "empty case grouping should not be flagged"
        );
    }

    #[test]
    fn test_allows_last_case_without_break() {
        let diags = lint("switch(x) { case 1: break; default: foo(); }");
        assert!(
            diags.is_empty(),
            "last case without break should not be flagged"
        );
    }
}
