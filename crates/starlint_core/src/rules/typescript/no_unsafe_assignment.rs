//! Rule: `typescript/no-unsafe-assignment`
//!
//! Disallow assigning `any` typed values. Assigning a value cast to `any`
//! silently removes type safety for the receiving binding, allowing type
//! errors to propagate undetected through the codebase.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `const x = expr as any` patterns.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, TSType};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations initialized with an `as any` assertion.
#[derive(Debug)]
pub struct NoUnsafeAssignment;

impl NativeRule for NoUnsafeAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-assignment".to_owned(),
            description: "Disallow assigning `any` typed values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclarator(decl) = kind else {
            return;
        };

        let Some(init) = &decl.init else {
            return;
        };

        if is_as_any(init) {
            ctx.report_warning(
                "typescript/no-unsafe-assignment",
                "Unsafe assignment — assigning an `as any` value removes type safety for this binding",
                Span::new(decl.span.start, decl.span.end),
            );
        }
    }
}

/// Check whether an expression is a `TSAsExpression` casting to `any`,
/// unwrapping parenthesized expressions.
fn is_as_any(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::TSAsExpression(as_expr) => {
            matches!(&as_expr.type_annotation, TSType::TSAnyKeyword(_))
        }
        Expression::ParenthesizedExpression(paren) => is_as_any(&paren.expression),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeAssignment)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_const_as_any() {
        let diags = lint("const x = value as any;");
        assert_eq!(diags.len(), 1, "`const x = value as any` should be flagged");
    }

    #[test]
    fn test_flags_let_as_any() {
        let diags = lint("let y = getData() as any;");
        assert_eq!(
            diags.len(),
            1,
            "`let y = getData() as any` should be flagged"
        );
    }

    #[test]
    fn test_flags_parenthesized_as_any() {
        let diags = lint("const z = (value as any);");
        assert_eq!(
            diags.len(),
            1,
            "parenthesized `as any` assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_as_string() {
        let diags = lint("const x = value as string;");
        assert!(diags.is_empty(), "`as string` should not be flagged");
    }

    #[test]
    fn test_allows_plain_assignment() {
        let diags = lint("const x = 42;");
        assert!(diags.is_empty(), "plain assignment should not be flagged");
    }
}
