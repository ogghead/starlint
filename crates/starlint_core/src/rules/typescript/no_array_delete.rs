//! Rule: `typescript/no-array-delete`
//!
//! Disallow using `delete` on array elements. Using `delete` on an array
//! creates a sparse array with a hole at that index, which is almost always
//! a bug. The length of the array is not updated and the element becomes
//! `undefined`. Use `Array.prototype.splice` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `delete arr[i]` expressions where the index is numeric, indicating
/// deletion from an array rather than an object.
#[derive(Debug)]
pub struct NoArrayDelete;

impl NativeRule for NoArrayDelete {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-array-delete".to_owned(),
            description: "Disallow using `delete` on array elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UnaryExpression(expr) = kind else {
            return;
        };

        if expr.operator != UnaryOperator::Delete {
            return;
        }

        // Only flag computed member expressions (bracket access) where the
        // index expression looks numeric — this distinguishes array element
        // deletion from dynamic object key deletion.
        let Expression::ComputedMemberExpression(member) = &expr.argument else {
            return;
        };

        if is_numeric_index(&member.expression) {
            ctx.report_error(
                "typescript/no-array-delete",
                "Do not `delete` array elements — it creates a sparse array hole. Use `splice` instead",
                Span::new(expr.span.start, expr.span.end),
            );
        }
    }
}

/// Check whether an expression looks like a numeric array index.
///
/// Returns `true` for numeric literals (`delete arr[0]`) and identifiers
/// commonly used as loop counters (`delete arr[i]`), which strongly suggest
/// array element deletion rather than object property deletion.
const fn is_numeric_index(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        // A bare identifier as index (e.g. `delete arr[i]`) is likely an
        // array index from a loop — flag conservatively.
        Expression::NumericLiteral(_) | Expression::Identifier(_)
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayDelete)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_delete_with_numeric_index() {
        let diags = lint("delete arr[0];");
        assert_eq!(
            diags.len(),
            1,
            "delete with numeric index should be flagged"
        );
    }

    #[test]
    fn test_flags_delete_with_variable_index() {
        let diags = lint("delete arr[i];");
        assert_eq!(
            diags.len(),
            1,
            "delete with variable index should be flagged"
        );
    }

    #[test]
    fn test_allows_delete_with_string_key() {
        let diags = lint("delete obj[\"key\"];");
        assert!(
            diags.is_empty(),
            "delete with string key should not be flagged"
        );
    }

    #[test]
    fn test_allows_delete_with_static_property() {
        let diags = lint("delete obj.prop;");
        assert!(
            diags.is_empty(),
            "delete with static property access should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_delete_array_access() {
        let diags = lint("let x = arr[0];");
        assert!(
            diags.is_empty(),
            "non-delete array access should not be flagged"
        );
    }
}
