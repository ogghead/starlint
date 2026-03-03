//! Rule: `prefer-bigint-literals`
//!
//! Prefer `BigInt` literals (`123n`) over `BigInt(123)` constructor calls
//! for literal arguments. The literal syntax is shorter and clearer.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `BigInt(literal)` calls — prefer `BigInt` literal syntax instead.
#[derive(Debug)]
pub struct PreferBigintLiterals;

impl NativeRule for PreferBigintLiterals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-bigint-literals".to_owned(),
            description: "Prefer `BigInt` literals over `BigInt()` constructor calls".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be a call to `BigInt`
        let Expression::Identifier(id) = &call.callee else {
            return;
        };

        if id.name.as_str() != "BigInt" {
            return;
        }

        // Must have exactly one argument
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        if call.arguments.len() != 1 {
            return;
        }

        if is_literal_bigint_candidate(first_arg) {
            ctx.report_warning(
                "prefer-bigint-literals",
                "Prefer `BigInt` literal syntax (e.g. `123n`) over `BigInt()` with a literal argument",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if an argument is a numeric literal or a string literal containing only digits.
fn is_literal_bigint_candidate(arg: &Argument<'_>) -> bool {
    match arg {
        Argument::NumericLiteral(_) => true,
        Argument::StringLiteral(lit) => {
            let val = lit.value.as_str();
            !val.is_empty() && val.chars().all(|c| c.is_ascii_digit())
        }
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferBigintLiterals)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bigint_with_numeric_literal() {
        let diags = lint("var x = BigInt(123);");
        assert_eq!(diags.len(), 1, "BigInt(123) should be flagged");
    }

    #[test]
    fn test_flags_bigint_with_string_digits() {
        let diags = lint("var x = BigInt(\"456\");");
        assert_eq!(diags.len(), 1, "BigInt with digit string should be flagged");
    }

    #[test]
    fn test_allows_bigint_literal() {
        let diags = lint("var x = 123n;");
        assert!(diags.is_empty(), "BigInt literal should not be flagged");
    }

    #[test]
    fn test_allows_bigint_with_variable() {
        let diags = lint("var x = BigInt(y);");
        assert!(
            diags.is_empty(),
            "BigInt with variable argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_bigint_with_non_digit_string() {
        let diags = lint("var x = BigInt(\"0xff\");");
        assert!(
            diags.is_empty(),
            "BigInt with non-digit string should not be flagged"
        );
    }

    #[test]
    fn test_allows_bigint_no_args() {
        let diags = lint("var x = BigInt();");
        assert!(
            diags.is_empty(),
            "BigInt with no arguments should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_function_call() {
        let diags = lint("var x = Number(123);");
        assert!(diags.is_empty(), "Number(123) should not be flagged");
    }
}
