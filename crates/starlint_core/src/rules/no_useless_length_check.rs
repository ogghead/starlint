//! Rule: `no-useless-length-check` (unicorn)
//!
//! Disallow useless `.length` checks before calling iteration methods.
//! For example, `if (arr.length > 0) { arr.forEach(...) }` is unnecessary
//! because `.forEach()` on an empty array is a no-op.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, Statement};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags useless `.length` checks before iteration methods.
#[derive(Debug)]
pub struct NoUselessLengthCheck;

/// Iteration methods that are no-ops on empty arrays.
const SAFE_ITERATION_METHODS: &[&str] = &[
    "forEach",
    "map",
    "filter",
    "some",
    "every",
    "find",
    "findIndex",
    "flatMap",
    "reduce",
    "reduceRight",
    "flat",
    "fill",
    "copyWithin",
    "entries",
    "keys",
    "values",
];

impl NativeRule for NoUselessLengthCheck {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-length-check".to_owned(),
            description: "Disallow useless .length check before iteration".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        // Check if the condition is `arr.length > 0` or `arr.length !== 0`
        // or `arr.length` (truthy check)
        let Some(array_name) = get_length_check_target(&if_stmt.test) else {
            return;
        };

        // Check if the body only contains iteration method calls on the same array
        if body_only_calls_iteration_method(&if_stmt.consequent, array_name) {
            ctx.report_warning(
                "no-useless-length-check",
                "The `.length` check is unnecessary; iteration methods are no-ops on empty arrays",
                Span::new(if_stmt.span.start, if_stmt.span.end),
            );
        }
    }
}

/// Extract the array name from a `.length` check expression.
fn get_length_check_target<'a>(expr: &'a Expression<'_>) -> Option<&'a str> {
    match expr {
        // `arr.length` (truthy check)
        Expression::StaticMemberExpression(member) if member.property.name == "length" => {
            if let Expression::Identifier(id) = &member.object {
                Some(id.name.as_str())
            } else {
                None
            }
        }
        // `arr.length > 0`, `arr.length !== 0`, etc.
        Expression::BinaryExpression(bin) => {
            let Expression::StaticMemberExpression(member) = &bin.left else {
                return None;
            };

            if member.property.name != "length" {
                return None;
            }

            let Expression::Identifier(id) = &member.object else {
                return None;
            };

            // Right side should be 0
            let Expression::NumericLiteral(num) = &bin.right else {
                return None;
            };

            #[allow(clippy::float_cmp)]
            (num.value == 0.0).then(|| id.name.as_str())
        }
        _ => None,
    }
}

/// Check if a statement body only calls iteration methods on the given array.
fn body_only_calls_iteration_method(stmt: &Statement<'_>, array_name: &str) -> bool {
    match stmt {
        Statement::BlockStatement(block) => {
            block.body.len() == 1
                && block
                    .body
                    .first()
                    .is_some_and(|s| body_only_calls_iteration_method(s, array_name))
        }
        Statement::ExpressionStatement(expr_stmt) => {
            is_iteration_call(&expr_stmt.expression, array_name)
        }
        _ => false,
    }
}

/// Check if an expression is `arr.forEach(...)`, `arr.map(...)`, etc.
fn is_iteration_call(expr: &Expression<'_>, array_name: &str) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };

    let Expression::Identifier(obj) = &member.object else {
        return false;
    };

    obj.name == array_name && SAFE_ITERATION_METHODS.contains(&member.property.name.as_str())
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessLengthCheck)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_length_check_before_foreach() {
        let diags = lint("if (arr.length > 0) { arr.forEach(fn); }");
        assert_eq!(
            diags.len(),
            1,
            "length check before forEach should be flagged"
        );
    }

    #[test]
    fn test_flags_truthy_length_check() {
        let diags = lint("if (arr.length) { arr.map(fn); }");
        assert_eq!(
            diags.len(),
            1,
            "truthy length check before map should be flagged"
        );
    }

    #[test]
    fn test_allows_length_check_with_other_code() {
        let diags = lint("if (arr.length > 0) { console.log('has items'); }");
        assert!(
            diags.is_empty(),
            "length check with non-iteration code should not be flagged"
        );
    }

    #[test]
    fn test_allows_without_length_check() {
        let diags = lint("arr.forEach(fn);");
        assert!(
            diags.is_empty(),
            "forEach without length check should not be flagged"
        );
    }
}
