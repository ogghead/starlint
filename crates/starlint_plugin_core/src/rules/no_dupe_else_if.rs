//! Rule: `no-dupe-else-if`
//!
//! Disallow duplicate conditions in if-else-if chains. Having the same
//! condition in multiple branches means the second branch is unreachable,
//! which is almost always a copy-paste error.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags duplicate conditions in if-else-if chains.
#[derive(Debug)]
pub struct NoDupeElseIf;

impl LintRule for NoDupeElseIf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-dupe-else-if".to_owned(),
            description: "Disallow duplicate conditions in if-else-if chains".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        // Collect duplicate spans first, then report (avoids borrow conflict).
        let duplicates = {
            let source = ctx.source_text();
            let mut seen: HashSet<String> = HashSet::new();
            let mut dupes: Vec<Span> = Vec::new();

            // Add the first condition
            let first_span = ctx.node(if_stmt.test).map_or(
                starlint_ast::types::Span::new(0, 0),
                starlint_ast::AstNode::span,
            );
            let first_start = usize::try_from(first_span.start).unwrap_or(0);
            let first_end = usize::try_from(first_span.end).unwrap_or(0);
            if let Some(text) = source.get(first_start..first_end) {
                seen.insert(text.to_owned());
            }

            // Walk the else-if chain
            let mut current_alt = if_stmt.alternate;
            while let Some(alt_id) = current_alt {
                if let Some(AstNode::IfStatement(else_if)) = ctx.node(alt_id) {
                    let test_span = ctx.node(else_if.test).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let test_start = usize::try_from(test_span.start).unwrap_or(0);
                    let test_end = usize::try_from(test_span.end).unwrap_or(0);
                    if let Some(text) = source.get(test_start..test_end) {
                        let key = text.to_owned();
                        if !seen.insert(key) {
                            dupes.push(Span::new(test_span.start, test_span.end));
                        }
                    }
                    current_alt = else_if.alternate;
                } else {
                    break;
                }
            }
            dupes
        };

        for span in duplicates {
            ctx.report(Diagnostic {
                rule_name: "no-dupe-else-if".to_owned(),
                message: "This branch can never execute because its condition is a duplicate"
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

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoDupeElseIf);

    #[test]
    fn test_flags_duplicate_else_if() {
        let diags = lint("if (a) {} else if (b) {} else if (a) {}");
        assert_eq!(
            diags.len(),
            1,
            "duplicate else-if condition should be flagged"
        );
    }

    #[test]
    fn test_allows_unique_conditions() {
        let diags = lint("if (a) {} else if (b) {} else if (c) {}");
        assert!(diags.is_empty(), "unique conditions should not be flagged");
    }

    #[test]
    fn test_flags_adjacent_duplicate() {
        let diags = lint("if (a) {} else if (a) {}");
        assert_eq!(
            diags.len(),
            1,
            "immediately duplicated condition should be flagged"
        );
    }

    #[test]
    fn test_allows_simple_if_else() {
        let diags = lint("if (a) {} else {}");
        assert!(diags.is_empty(), "simple if-else should not be flagged");
    }

    #[test]
    fn test_allows_different_expressions() {
        let diags = lint("if (x > 0) {} else if (x < 0) {} else if (x === 0) {}");
        assert!(
            diags.is_empty(),
            "different expressions should not be flagged"
        );
    }
}
