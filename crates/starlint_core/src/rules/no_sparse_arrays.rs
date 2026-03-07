//! Rule: `no-sparse-arrays`
//!
//! Disallow sparse arrays (arrays with empty slots like `[1,,3]`).
//! Sparse arrays are confusing because the empty slots are `undefined`
//! but behave differently from explicit `undefined` values.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags array literals containing empty slots (elisions).
#[derive(Debug)]
pub struct NoSparseArrays;

impl LintRule for NoSparseArrays {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-sparse-arrays".to_owned(),
            description: "Disallow sparse arrays".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrayExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ArrayExpression(arr) = node else {
            return;
        };

        // Detect sparse arrays by scanning source text for elision patterns
        // (consecutive commas or leading commas within the array literal)
        let source = ctx.source_text();
        let start = arr.span.start as usize;
        let end = arr.span.end as usize;
        let Some(arr_text) = source.get(start..end) else {
            return;
        };

        // Check for sparse patterns: ",," or "[," (leading elision)
        let has_elision = {
            let inner = arr_text.trim();
            // Strip the outer brackets
            let inner = inner.strip_prefix('[').unwrap_or(inner);
            let inner = inner.strip_suffix(']').unwrap_or(inner);
            let trimmed = inner.trim();
            // Leading comma means leading elision
            trimmed.starts_with(',')
                // Or consecutive commas (with optional whitespace between)
                || trimmed.contains(",,")
        };

        if has_elision {
            ctx.report(Diagnostic {
                rule_name: "no-sparse-arrays".to_owned(),
                message: "Unexpected comma in middle of array (sparse array)".to_owned(),
                span: Span::new(arr.span.start, arr.span.end),
                severity: Severity::Error,
                help: Some("Replace empty slots with explicit `undefined`".to_owned()),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoSparseArrays)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_sparse_array() {
        let diags = lint("const a = [1,,3];");
        assert_eq!(diags.len(), 1, "sparse array should be flagged");
    }

    #[test]
    fn test_flags_leading_elision() {
        let diags = lint("const a = [,1,2];");
        assert_eq!(diags.len(), 1, "leading elision should be flagged");
    }

    #[test]
    fn test_flags_trailing_elision_in_middle() {
        let diags = lint("const a = [1,,,4];");
        assert_eq!(
            diags.len(),
            1,
            "multiple elisions should be flagged once per array"
        );
    }

    #[test]
    fn test_allows_normal_array() {
        let diags = lint("const a = [1, 2, 3];");
        assert!(diags.is_empty(), "normal array should not be flagged");
    }

    #[test]
    fn test_allows_empty_array() {
        let diags = lint("const a = [];");
        assert!(diags.is_empty(), "empty array should not be flagged");
    }

    #[test]
    fn test_allows_array_with_undefined() {
        let diags = lint("const a = [undefined, undefined];");
        assert!(diags.is_empty(), "explicit undefined should not be flagged");
    }
}
