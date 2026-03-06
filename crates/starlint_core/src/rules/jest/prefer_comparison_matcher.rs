//! Rule: `jest/prefer-comparison-matcher`
//!
//! Suggest `toBeGreaterThan(x)` / `toBeLessThan(x)` etc. over
//! `expect(a > b).toBe(true)`. The dedicated comparison matchers provide
//! better failure messages showing actual and expected values.

use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(a > b).toBe(true)` patterns that could use comparison matchers.
#[derive(Debug)]
pub struct PreferComparisonMatcher;

impl NativeRule for PreferComparisonMatcher {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-comparison-matcher".to_owned(),
            description: "Suggest using comparison matchers instead of `expect(a > b).toBe(true)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `.toBe(true)` or `.toBe(false)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };
        let is_bool = matches!(arg_expr, Expression::BooleanLiteral(_));
        if !is_bool {
            return;
        }

        // Object must be `expect(...)` call
        let Expression::CallExpression(expect_call) = &member.object else {
            return;
        };
        let is_expect = matches!(
            &expect_call.callee,
            Expression::Identifier(id) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // First arg of expect() must be a comparison binary expression
        let Some(expect_arg) = expect_call.arguments.first() else {
            return;
        };
        let Some(expect_arg_expr) = expect_arg.as_expression() else {
            return;
        };
        let Expression::BinaryExpression(binary) = expect_arg_expr else {
            return;
        };

        let suggestion = match binary.operator {
            BinaryOperator::GreaterThan => "toBeGreaterThan",
            BinaryOperator::GreaterEqualThan => "toBeGreaterThanOrEqual",
            BinaryOperator::LessThan => "toBeLessThan",
            BinaryOperator::LessEqualThan => "toBeLessThanOrEqual",
            _ => return,
        };

        // Build fix: extract left/right operands and construct matcher call
        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let left_span = binary.left.span();
            let right_span = binary.right.span();
            let left_text = source
                .get(left_span.start as usize..left_span.end as usize)
                .unwrap_or("");
            let right_text = source
                .get(right_span.start as usize..right_span.end as usize)
                .unwrap_or("");
            let is_true = matches!(arg_expr, Expression::BooleanLiteral(b) if b.value);
            let negated = if is_true { "" } else { ".not" };
            let replacement = format!("expect({left_text}){negated}.{suggestion}({right_text})");
            (!left_text.is_empty() && !right_text.is_empty()).then(|| Fix {
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement,
                }],
            })
        };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-comparison-matcher".to_owned(),
            message: format!(
                "Use `{suggestion}()` instead of `expect(a {op} b).{method}(true/false)`",
                op = operator_str(binary.operator),
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

/// Get the string representation of a comparison operator.
const fn operator_str(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::GreaterEqualThan => ">=",
        BinaryOperator::LessThan => "<",
        BinaryOperator::LessEqualThan => "<=",
        _ => "?",
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferComparisonMatcher)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_greater_than() {
        let diags = lint("expect(a > b).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(a > b).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_less_equal() {
        let diags = lint("expect(a <= b).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(a <= b).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_allows_comparison_matcher() {
        let diags = lint("expect(a).toBeGreaterThan(b);");
        assert!(
            diags.is_empty(),
            "`toBeGreaterThan()` should not be flagged"
        );
    }
}
