//! Rule: `typescript/no-extra-non-null-assertion`
//!
//! Disallow extra non-null assertions. Writing `x!!` applies two `!` postfix
//! operators, but the second assertion is always redundant — if `x!` is
//! non-null, asserting it again adds no value and suggests a typo.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `TSNonNullExpression` nodes whose inner expression is also a
/// `TSNonNullExpression` (i.e. `x!!`).
#[derive(Debug)]
pub struct NoExtraNonNullAssertion;

impl NativeRule for NoExtraNonNullAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-extra-non-null-assertion".to_owned(),
            description: "Disallow extra non-null assertions (`!!`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSNonNullExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSNonNullExpression(expr) = kind else {
            return;
        };

        if matches!(&expr.expression, Expression::TSNonNullExpression(_)) {
            ctx.report_error(
                "typescript/no-extra-non-null-assertion",
                "Extra non-null assertion — `x!` is sufficient, `x!!` is redundant",
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraNonNullAssertion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_double_non_null() {
        let diags = lint("declare const x: string | null; x!!;");
        assert_eq!(diags.len(), 1, "`x!!` should be flagged");
    }

    #[test]
    fn test_flags_double_non_null_with_member_access() {
        let diags = lint("declare const x: { foo: string } | null; x!!.foo;");
        assert_eq!(diags.len(), 1, "`x!!.foo` should be flagged");
    }

    #[test]
    fn test_allows_single_non_null() {
        let diags = lint("declare const x: string | null; x!;");
        assert!(diags.is_empty(), "single `x!` should not be flagged");
    }

    #[test]
    fn test_allows_single_non_null_with_member_access() {
        let diags = lint("declare const x: { foo: string } | null; x!.foo;");
        assert!(diags.is_empty(), "`x!.foo` should not be flagged");
    }
}
