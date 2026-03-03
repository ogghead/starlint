//! Rule: `no-sparse-arrays`
//!
//! Disallow sparse arrays (arrays with empty slots like `[1,,3]`).
//! Sparse arrays are confusing because the empty slots are `undefined`
//! but behave differently from explicit `undefined` values.

use oxc_ast::AstKind;
use oxc_ast::ast::ArrayExpressionElement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags array literals containing empty slots (elisions).
#[derive(Debug)]
pub struct NoSparseArrays;

impl NativeRule for NoSparseArrays {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-sparse-arrays".to_owned(),
            description: "Disallow sparse arrays".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrayExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ArrayExpression(arr) = kind else {
            return;
        };

        let has_elision = arr
            .elements
            .iter()
            .any(|el| matches!(el, ArrayExpressionElement::Elision(_)));

        if has_elision {
            ctx.report_error(
                "no-sparse-arrays",
                "Unexpected comma in middle of array (sparse array)",
                Span::new(arr.span.start, arr.span.end),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSparseArrays)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
