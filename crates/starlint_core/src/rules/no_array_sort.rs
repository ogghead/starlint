//! Rule: `no-array-sort`
//!
//! Disallow `Array.prototype.sort()` which mutates the array in-place.
//! Prefer `toSorted()` which returns a new sorted array without modifying
//! the original.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.sort()` calls that mutate arrays in-place.
#[derive(Debug)]
pub struct NoArraySort;

impl LintRule for NoArraySort {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-sort".to_owned(),
            description: "Disallow `.sort()` which mutates the array — prefer `.toSorted()`"
                .to_owned(),
            category: Category::Suggestion,
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "sort" {
            return;
        }

        // Fix: replace `.sort` with `.toSorted` in the property name
        let prop_len = u32::try_from(member.property.len()).unwrap_or(0);
        let prop_end = member.span.end;
        let prop_start = prop_end.saturating_sub(prop_len);
        let fix = Some(Fix {
            kind: FixKind::SuggestionFix,
            message: "Replace `.sort()` with `.toSorted()`".to_owned(),
            edits: vec![Edit {
                span: Span::new(prop_start, prop_end),
                replacement: "toSorted".to_owned(),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "no-array-sort".to_owned(),
            message: "`.sort()` mutates the array in-place — prefer `.toSorted()` instead"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace `.sort()` with `.toSorted()`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArraySort)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_sort_no_args() {
        let diags = lint("arr.sort();");
        assert_eq!(diags.len(), 1, ".sort() should be flagged");
    }

    #[test]
    fn test_flags_sort_with_comparator() {
        let diags = lint("arr.sort((a, b) => a - b);");
        assert_eq!(
            diags.len(),
            1,
            ".sort() with comparator should also be flagged (still mutates)"
        );
    }

    #[test]
    fn test_flags_spread_sort() {
        let diags = lint("[...arr].sort();");
        assert_eq!(
            diags.len(),
            1,
            "[...arr].sort() is still a .sort() call and should be flagged"
        );
    }

    #[test]
    fn test_allows_to_sorted() {
        let diags = lint("arr.toSorted();");
        assert!(diags.is_empty(), ".toSorted() should not be flagged");
    }

    #[test]
    fn test_allows_to_sorted_with_comparator() {
        let diags = lint("arr.toSorted((a, b) => a - b);");
        assert!(
            diags.is_empty(),
            ".toSorted() with comparator should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("arr.map(x => x);");
        assert!(
            diags.is_empty(),
            "unrelated method calls should not be flagged"
        );
    }
}
