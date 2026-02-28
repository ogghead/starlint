//! Rule: `typescript/no-dynamic-delete`
//!
//! Disallow `delete` with computed key expressions. Using `delete` with a
//! dynamic (bracket-accessed) key makes code harder to reason about and
//! prevents certain engine optimizations. Use `Map` or `Set` for dynamic
//! key collections, or `Reflect.deleteProperty` for explicit intent.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `delete` expressions that use computed (bracket) member access.
#[derive(Debug)]
pub struct NoDynamicDelete;

impl NativeRule for NoDynamicDelete {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-dynamic-delete".to_owned(),
            description: "Disallow `delete` with computed key expressions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        // Only flag when the operand is a computed member expression (bracket access).
        // `delete obj.prop` (static access) is fine.
        if matches!(
            &expr.argument,
            Expression::ComputedMemberExpression(_)
        ) {
            ctx.report_warning(
                "typescript/no-dynamic-delete",
                "Do not `delete` dynamically computed keys — use `Map` or `Set` instead",
                Span::new(expr.span.start, expr.span.end),
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDynamicDelete)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_delete_with_variable_key() {
        let diags = lint("delete obj[key];");
        assert_eq!(
            diags.len(),
            1,
            "delete with dynamic key should be flagged"
        );
    }

    #[test]
    fn test_flags_delete_with_string_key() {
        let diags = lint("delete obj[\"key\"];");
        assert_eq!(
            diags.len(),
            1,
            "delete with string bracket key should be flagged"
        );
    }

    #[test]
    fn test_allows_delete_with_static_property() {
        let diags = lint("delete obj.key;");
        assert!(
            diags.is_empty(),
            "delete with static property access should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_delete_computed_access() {
        let diags = lint("obj[key];");
        assert!(
            diags.is_empty(),
            "non-delete computed access should not be flagged"
        );
    }
}
