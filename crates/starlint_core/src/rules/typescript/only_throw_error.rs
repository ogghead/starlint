//! Rule: `typescript/only-throw-error`
//!
//! Disallow throwing non-Error values. Flags `throw` statements where the
//! argument is a literal (string, number, boolean, null, undefined) rather
//! than an Error object or variable that likely holds one.
//!
//! Simplified syntax-only version — full checking requires type information.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/only-throw-error";

/// Flags `throw` statements that throw literal values instead of Error objects.
#[derive(Debug)]
pub struct OnlyThrowError;

impl NativeRule for OnlyThrowError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow throwing non-Error values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ThrowStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ThrowStatement(throw) = kind else {
            return;
        };

        if is_non_error_literal(&throw.argument) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Expected an Error object to be thrown — do not throw literals".to_owned(),
                span: Span::new(throw.span.start, throw.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the expression is a literal value that is not an Error:
/// string, number, boolean, null, undefined, bigint.
fn is_non_error_literal(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::TemplateLiteral(_)
    ) || is_undefined_identifier(expr)
}

/// Returns `true` if the expression is the identifier `undefined`.
fn is_undefined_identifier(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "undefined")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(OnlyThrowError)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_throw_string() {
        let diags = lint("throw \"error message\";");
        assert_eq!(
            diags.len(),
            1,
            "throwing a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_number() {
        let diags = lint("throw 42;");
        assert_eq!(
            diags.len(),
            1,
            "throwing a number literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_boolean() {
        let diags = lint("throw true;");
        assert_eq!(
            diags.len(),
            1,
            "throwing a boolean literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_null() {
        let diags = lint("throw null;");
        assert_eq!(diags.len(), 1, "throwing null should be flagged");
    }

    #[test]
    fn test_flags_throw_undefined() {
        let diags = lint("throw undefined;");
        assert_eq!(diags.len(), 1, "throwing undefined should be flagged");
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('something went wrong');");
        assert!(diags.is_empty(), "throwing new Error should not be flagged");
    }

    #[test]
    fn test_allows_throw_variable() {
        let diags = lint("throw err;");
        assert!(
            diags.is_empty(),
            "throwing a variable should not be flagged"
        );
    }
}
