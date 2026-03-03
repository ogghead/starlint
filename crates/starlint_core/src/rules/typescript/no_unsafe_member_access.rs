//! Rule: `typescript/no-unsafe-member-access`
//!
//! Disallow member access on `any` typed values. Accessing a property on
//! a value cast to `any` silently produces another `any`, allowing type
//! unsafety to cascade through the codebase without compiler warnings.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `(expr as any).property` patterns.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, TSType};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags static member access on expressions cast to `any`.
#[derive(Debug)]
pub struct NoUnsafeMemberAccess;

impl NativeRule for NoUnsafeMemberAccess {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-member-access".to_owned(),
            description: "Disallow member access on `any` typed values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StaticMemberExpression(member) = kind else {
            return;
        };

        if is_as_any_object(&member.object) {
            ctx.report_warning(
                "typescript/no-unsafe-member-access",
                "Unsafe member access — accessing a property on an `as any` expression propagates type unsafety",
                Span::new(member.span.start, member.span.end),
            );
        }
    }
}

/// Check whether an object expression is an `as any` cast, unwrapping
/// parenthesized expressions.
fn is_as_any_object(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::TSAsExpression(as_expr) => {
            matches!(&as_expr.type_annotation, TSType::TSAnyKeyword(_))
        }
        Expression::ParenthesizedExpression(paren) => is_as_any_object(&paren.expression),
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeMemberAccess)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_member_access_on_as_any() {
        let diags = lint("let x = (value as any).foo;");
        assert_eq!(diags.len(), 1, "`(value as any).foo` should be flagged");
    }

    #[test]
    fn test_flags_nested_member_access() {
        let diags = lint("let x = (getData() as any).result;");
        assert_eq!(
            diags.len(),
            1,
            "`(getData() as any).result` should be flagged"
        );
    }

    #[test]
    fn test_allows_typed_member_access() {
        let diags = lint("let x = (value as Record<string, number>).foo;");
        assert!(
            diags.is_empty(),
            "member access on typed assertion should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_member_access() {
        let diags = lint("let x = obj.foo;");
        assert!(
            diags.is_empty(),
            "plain member access should not be flagged"
        );
    }

    #[test]
    fn test_flags_parenthesized_as_any_member() {
        let diags = lint("let x = ((value as any)).bar;");
        assert_eq!(
            diags.len(),
            1,
            "double-parenthesized `as any` member access should be flagged"
        );
    }
}
