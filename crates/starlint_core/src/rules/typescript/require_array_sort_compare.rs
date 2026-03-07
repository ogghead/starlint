//! Rule: `typescript/require-array-sort-compare`
//!
//! Require a compare function argument in `Array.prototype.sort()`. Without a
//! compare function, `sort()` converts elements to strings and sorts them
//! lexicographically, which is often not what you want for numeric or complex
//! data.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.sort()` calls with zero arguments.
#[derive(Debug)]
pub struct RequireArraySortCompare;

impl LintRule for RequireArraySortCompare {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/require-array-sort-compare".to_owned(),
            description: "Require a compare function argument in `Array.prototype.sort()`"
                .to_owned(),
            category: Category::Correctness,
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

        let is_sort = matches!(
            ctx.node(call.callee),
            Some(AstNode::StaticMemberExpression(member)) if member.property.as_str() == "sort"
        );

        if !is_sort {
            return;
        }

        if call.arguments.is_empty() {
            ctx.report(Diagnostic {
                rule_name: "typescript/require-array-sort-compare".to_owned(),
                message: "Provide a compare function to `.sort()` — without one, elements are sorted as strings".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireArraySortCompare)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_sort_without_compare() {
        let diags = lint("const arr = [3, 1, 2]; arr.sort();");
        assert_eq!(
            diags.len(),
            1,
            "`.sort()` without a compare function should be flagged"
        );
    }

    #[test]
    fn test_allows_sort_with_compare() {
        let diags = lint("const arr = [3, 1, 2]; arr.sort((a, b) => a - b);");
        assert!(
            diags.is_empty(),
            "`.sort()` with a compare function should not be flagged"
        );
    }

    #[test]
    fn test_allows_sort_with_named_compare() {
        let diags = lint("const arr = [3, 1, 2]; arr.sort(compareFn);");
        assert!(
            diags.is_empty(),
            "`.sort()` with a named compare function should not be flagged"
        );
    }

    #[test]
    fn test_ignores_non_sort_method() {
        let diags = lint("arr.filter();");
        assert!(
            diags.is_empty(),
            "non-sort method calls should not be flagged"
        );
    }

    #[test]
    fn test_flags_chained_sort_without_compare() {
        let diags = lint("getItems().sort();");
        assert_eq!(
            diags.len(),
            1,
            "chained `.sort()` without compare should be flagged"
        );
    }
}
