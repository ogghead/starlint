//! Rule: `number-arg-out-of-range` (OXC)
//!
//! Detect calls to functions like `parseInt(x, radix)` or `Number.toFixed(n)`
//! where the numeric argument is outside the valid range.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags numeric arguments outside valid ranges.
#[derive(Debug)]
pub struct NumberArgOutOfRange;

impl NativeRule for NumberArgOutOfRange {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "number-arg-out-of-range".to_owned(),
            description: "Detect numeric arguments outside valid ranges".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let findings = check_parse_int(call)
            .or_else(|| check_method_range(call, "toFixed", 0, 100))
            .or_else(|| check_method_range(call, "toPrecision", 1, 100))
            .or_else(|| check_method_range(call, "toExponential", 0, 100));

        if let Some(message) = findings {
            ctx.report(Diagnostic {
                rule_name: "number-arg-out-of-range".to_owned(),
                message,
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check parseInt radix (must be 2-36).
fn check_parse_int(call: &oxc_ast::ast::CallExpression<'_>) -> Option<String> {
    let is_parse_int = match &call.callee {
        Expression::Identifier(id) => id.name.as_str() == "parseInt",
        Expression::StaticMemberExpression(member) => {
            member.property.name.as_str() == "parseInt"
                && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Number")
        }
        _ => false,
    };

    if !is_parse_int {
        return None;
    }

    let radix_arg = call.arguments.get(1)?;
    let radix = get_integer_value(radix_arg.as_expression()?)?;

    if !(2..=36).contains(&radix) {
        return Some(format!(
            "`parseInt()` radix must be between 2 and 36, got {radix}"
        ));
    }

    None
}

/// Check a method call's first argument against a valid range.
fn check_method_range(
    call: &oxc_ast::ast::CallExpression<'_>,
    method_name: &str,
    min: i64,
    max: i64,
) -> Option<String> {
    let is_method = matches!(
        &call.callee,
        Expression::StaticMemberExpression(member) if member.property.name.as_str() == method_name
    );

    if !is_method {
        return None;
    }

    let arg = call.arguments.first()?;
    let value = get_integer_value(arg.as_expression()?)?;

    if !(min..=max).contains(&value) {
        return Some(format!(
            "`.{method_name}()` argument must be between {min} and {max}, got {value}"
        ));
    }

    None
}

/// Get integer value from a numeric literal expression.
///
/// Returns `None` if the expression is not a numeric literal or if the value
/// cannot be safely converted to i64.
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
fn get_integer_value(expr: &Expression<'_>) -> Option<i64> {
    match expr {
        Expression::NumericLiteral(n) => {
            let val = n.value;
            // Only consider integer values (no fractional part) within i64 range
            #[allow(clippy::float_cmp)]
            let is_integer = val == val.trunc() && val >= i64::MIN as f64 && val <= i64::MAX as f64;
            is_integer.then_some(val as i64)
        }
        _ => None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NumberArgOutOfRange)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_parse_int_bad_radix() {
        let diags = lint("parseInt('10', 37);");
        assert_eq!(diags.len(), 1, "parseInt with radix 37 should be flagged");
    }

    #[test]
    fn test_flags_parse_int_radix_zero() {
        let diags = lint("parseInt('10', 0);");
        assert_eq!(diags.len(), 1, "parseInt with radix 0 should be flagged");
    }

    #[test]
    fn test_allows_parse_int_valid_radix() {
        let diags = lint("parseInt('10', 16);");
        assert!(
            diags.is_empty(),
            "parseInt with radix 16 should not be flagged"
        );
    }

    #[test]
    fn test_flags_to_fixed_out_of_range() {
        let diags = lint("n.toFixed(101);");
        assert_eq!(diags.len(), 1, "toFixed(101) should be flagged");
    }

    #[test]
    fn test_allows_to_fixed_valid() {
        let diags = lint("n.toFixed(2);");
        assert!(diags.is_empty(), "toFixed(2) should not be flagged");
    }
}
