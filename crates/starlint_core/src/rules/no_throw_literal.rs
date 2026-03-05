//! Rule: `no-throw-literal`
//!
//! Restrict what can be thrown as an exception. Only `Error` objects (or
//! subclasses) should be thrown because they capture a stack trace.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `throw` statements that throw non-Error values.
#[derive(Debug)]
pub struct NoThrowLiteral;

impl NativeRule for NoThrowLiteral {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-throw-literal".to_owned(),
            description: "Disallow throwing literals and non-Error objects".to_owned(),
            category: Category::Style,
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

        if is_literal_or_non_error(&throw.argument) {
            ctx.report(Diagnostic {
                rule_name: "no-throw-literal".to_owned(),
                message: "Expected an Error object to be thrown".to_owned(),
                span: Span::new(throw.span.start, throw.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a literal value (string, number, boolean, null,
/// undefined) rather than an Error object.
const fn is_literal_or_non_error(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThrowLiteral)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_throw_string() {
        let diags = lint("throw 'error';");
        assert_eq!(
            diags.len(),
            1,
            "throwing a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_number() {
        let diags = lint("throw 0;");
        assert_eq!(diags.len(), 1, "throwing a number should be flagged");
    }

    #[test]
    fn test_flags_throw_null() {
        let diags = lint("throw null;");
        assert_eq!(diags.len(), 1, "throwing null should be flagged");
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('msg');");
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

    #[test]
    fn test_allows_throw_call() {
        let diags = lint("throw getError();");
        assert!(
            diags.is_empty(),
            "throwing a function call should not be flagged"
        );
    }
}
