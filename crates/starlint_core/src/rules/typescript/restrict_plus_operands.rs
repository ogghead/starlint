//! Rule: `typescript/restrict-plus-operands`
//!
//! Disallow the `+` operator with mixed string and number literal operands.
//! Adding a string literal to a number literal (or vice versa) is almost always
//! a mistake — the number is silently coerced to a string, producing unexpected
//! concatenation instead of arithmetic.
//!
//! Simplified syntax-only version — full checking requires type information.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/restrict-plus-operands";

/// Flags `+` expressions where one operand is a string literal and the other
/// is a numeric literal.
#[derive(Debug)]
pub struct RestrictPlusOperands;

impl NativeRule for RestrictPlusOperands {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `+` operator with mixed string and number literal operands"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BinaryExpression(bin) = kind else {
            return;
        };

        if bin.operator != BinaryOperator::Addition {
            return;
        }

        if is_mixed_string_number(&bin.left, &bin.right) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Unexpected mixed string and number operands for `+` — the number will be \
                 coerced to a string"
                        .to_owned(),
                span: Span::new(bin.span.start, bin.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if one operand is a string literal and the other is a
/// numeric literal (in either order).
const fn is_mixed_string_number(left: &Expression<'_>, right: &Expression<'_>) -> bool {
    let left_is_string = matches!(left, Expression::StringLiteral(_));
    let left_is_number = matches!(left, Expression::NumericLiteral(_));
    let right_is_string = matches!(right, Expression::StringLiteral(_));
    let right_is_number = matches!(right, Expression::NumericLiteral(_));

    (left_is_string && right_is_number) || (left_is_number && right_is_string)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RestrictPlusOperands)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_plus_number() {
        let diags = lint(r#"const x = "hello" + 42;"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal + number literal should be flagged"
        );
    }

    #[test]
    fn test_flags_number_plus_string() {
        let diags = lint(r#"const x = 42 + "hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "number literal + string literal should be flagged"
        );
    }

    #[test]
    fn test_allows_string_plus_string() {
        let diags = lint(r#"const x = "hello" + " world";"#);
        assert!(
            diags.is_empty(),
            "string + string concatenation should not be flagged"
        );
    }

    #[test]
    fn test_allows_number_plus_number() {
        let diags = lint("const x = 1 + 2;");
        assert!(
            diags.is_empty(),
            "number + number arithmetic should not be flagged"
        );
    }

    #[test]
    fn test_allows_variable_plus_number() {
        let diags = lint("const x = y + 42;");
        assert!(
            diags.is_empty(),
            "variable + number should not be flagged without type info"
        );
    }
}
