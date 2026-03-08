//! Rule: `guard-for-in`
//!
//! Require `hasOwnProperty` checks in `for-in` loops. The `for-in` statement
//! iterates over all enumerable properties of an object, including inherited
//! ones. It is a common best practice to filter out inherited properties with
//! an `if` guard (e.g. `if (obj.hasOwnProperty(k))`) or a `continue` guard.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `for-in` loops that do not guard with an `if` statement or `continue`.
#[derive(Debug)]
pub struct GuardForIn;

impl LintRule for GuardForIn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "guard-for-in".to_owned(),
            description: "Require `hasOwnProperty` check in `for-in` loops".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ForInStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ForInStatement(for_in) = node else {
            return;
        };

        if is_guarded(for_in.body, ctx) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "guard-for-in".to_owned(),
            message: "The body of a `for-in` should be wrapped in an `if` statement to filter unwanted properties from the prototype".to_owned(),
            span: Span::new(for_in.span.start, for_in.span.end),
            severity: Severity::Warning,
            help: Some("Add `if (obj.hasOwnProperty(key))` guard or use `Object.keys()` instead".to_owned()),
            fix: None,
            labels: vec![],
        });
    }
}

/// Check if the for-in body is guarded.
///
/// A body is considered guarded if:
/// - It is a block whose first statement is an `if` statement (guard pattern), OR
/// - It is a block whose only statement is a `continue` statement, OR
/// - It is directly an `if` statement (no block wrapper)
fn is_guarded(body: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(body_node) = ctx.node(body) else {
        return false;
    };
    match body_node {
        AstNode::BlockStatement(block) => {
            let stmts = &block.body;

            // Empty block — nothing to guard
            if stmts.is_empty() {
                return true;
            }

            // First statement is an if-statement — accepted as a guard
            if let Some(first_id) = stmts.first() {
                if let Some(AstNode::IfStatement(_)) = ctx.node(*first_id) {
                    return true;
                }
            }

            // Single continue statement — accepted as a guard
            if stmts.len() == 1 {
                if let Some(first_id) = stmts.first() {
                    if let Some(AstNode::ContinueStatement(_)) = ctx.node(*first_id) {
                        return true;
                    }
                }
            }

            false
        }
        // If statement directly as body (no block), or empty statement — nothing to guard
        AstNode::IfStatement(_) | AstNode::EmptyStatement(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(GuardForIn)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_unguarded_for_in() {
        let diags = lint("for (var k in obj) { use(k); }");
        assert_eq!(diags.len(), 1, "for-in without guard should be flagged");
    }

    #[test]
    fn test_allows_if_guard() {
        let diags = lint("for (var k in obj) { if (obj.hasOwnProperty(k)) { use(k); } }");
        assert!(
            diags.is_empty(),
            "for-in with if guard should not be flagged"
        );
    }

    #[test]
    fn test_allows_if_continue_guard() {
        let diags = lint("for (var k in obj) { if (!obj.hasOwnProperty(k)) continue; use(k); }");
        assert!(
            diags.is_empty(),
            "for-in with if-continue guard should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_continue() {
        let diags = lint("for (var k in obj) { continue; }");
        assert!(
            diags.is_empty(),
            "for-in with only continue should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_body() {
        let diags = lint("for (var k in obj) { }");
        assert!(
            diags.is_empty(),
            "for-in with empty body should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_statements_no_guard() {
        let diags = lint("for (var k in obj) { foo(k); bar(k); }");
        assert_eq!(
            diags.len(),
            1,
            "for-in with multiple unguarded statements should be flagged"
        );
    }
}
