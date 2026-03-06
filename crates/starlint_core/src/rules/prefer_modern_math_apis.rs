//! Rule: `prefer-modern-math-apis` (unicorn)
//!
//! Prefer modern `Math` APIs over legacy patterns. For example:
//! - `Math.log(x) / Math.log(2)` → `Math.log2(x)`
//! - `Math.log(x) / Math.log(10)` → `Math.log10(x)`
//! - `Math.pow(x, 0.5)` → `Math.sqrt(x)` / `Math.cbrt(x)`

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags legacy Math patterns that have modern equivalents.
#[derive(Debug)]
pub struct PreferModernMathApis;

impl NativeRule for PreferModernMathApis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-modern-math-apis".to_owned(),
            description: "Prefer modern Math APIs".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BinaryExpression, AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // Check for Math.log(x) / Math.log(base)
            AstKind::BinaryExpression(bin) => {
                if !matches!(bin.operator, oxc_ast::ast::BinaryOperator::Division) {
                    return;
                }

                if is_math_method_call(&bin.left, "log") && is_math_method_call(&bin.right, "log") {
                    if let Some((method, suggestion)) = get_log_suggestion_with_method(&bin.right) {
                        // Extract the argument from the numerator Math.log(x)
                        let fix = extract_math_log_arg_text(ctx.source_text(), &bin.left).map(
                            |arg_text| Fix {
                                kind: FixKind::SafeFix,
                                message: format!("Replace with `Math.{method}({arg_text})`"),
                                edits: vec![Edit {
                                    span: Span::new(bin.span.start, bin.span.end),
                                    replacement: format!("Math.{method}({arg_text})"),
                                }],
                                is_snippet: false,
                            },
                        );

                        ctx.report(Diagnostic {
                            rule_name: "prefer-modern-math-apis".to_owned(),
                            message: format!(
                                "Prefer `{suggestion}` over `Math.log(x) / Math.log(base)`"
                            ),
                            span: Span::new(bin.span.start, bin.span.end),
                            severity: Severity::Warning,
                            help: Some(format!("Replace with `{suggestion}`")),
                            fix,
                            labels: vec![],
                        });
                    }
                }
            }
            // Check for Math.pow(x, 0.5)
            AstKind::CallExpression(call) => {
                if !is_math_member_callee(&call.callee, "pow") {
                    return;
                }

                if call.arguments.len() != 2 {
                    return;
                }

                let Some(second_arg) = call.arguments.get(1) else {
                    return;
                };

                if let oxc_ast::ast::Argument::NumericLiteral(num) = second_arg {
                    #[allow(clippy::float_cmp)]
                    if num.value == 0.5 {
                        // Extract the first argument source text
                        let fix = call.arguments.first().and_then(|first_arg| {
                            let arg_span = first_arg.span();
                            let arg_text = ctx
                                .source_text()
                                .get(arg_span.start as usize..arg_span.end as usize)?
                                .to_owned();
                            (!arg_text.is_empty()).then(|| Fix {
                                kind: FixKind::SafeFix,
                                message: format!("Replace with `Math.sqrt({arg_text})`"),
                                edits: vec![Edit {
                                    span: Span::new(call.span.start, call.span.end),
                                    replacement: format!("Math.sqrt({arg_text})"),
                                }],
                                is_snippet: false,
                            })
                        });

                        ctx.report(Diagnostic {
                            rule_name: "prefer-modern-math-apis".to_owned(),
                            message: "Prefer `Math.sqrt(x)` over `Math.pow(x, 0.5)`".to_owned(),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Warning,
                            help: Some("Replace with `Math.sqrt(x)`".to_owned()),
                            fix,
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if an expression is `Math.method(...)`.
fn is_math_method_call(expr: &Expression<'_>, method: &str) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };

    let Expression::Identifier(obj) = &member.object else {
        return false;
    };

    obj.name == "Math" && member.property.name == method
}

/// Check if an expression is `Math.method` (as a callee, not wrapped in a call).
fn is_math_member_callee(expr: &Expression<'_>, method: &str) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };

    let Expression::Identifier(obj) = &member.object else {
        return false;
    };

    obj.name == "Math" && member.property.name == method
}

/// Get the method name and suggestion for `Math.log(x) / Math.log(base)` patterns.
/// Returns `(method_name, full_suggestion)`.
fn get_log_suggestion_with_method(
    divisor: &Expression<'_>,
) -> Option<(&'static str, &'static str)> {
    let Expression::CallExpression(call) = divisor else {
        return None;
    };

    let first_arg = call.arguments.first()?;

    let oxc_ast::ast::Argument::NumericLiteral(num) = first_arg else {
        return None;
    };

    #[allow(clippy::float_cmp)]
    if num.value == 2.0 {
        Some(("log2", "Math.log2(x)"))
    } else if num.value == 10.0 {
        Some(("log10", "Math.log10(x)"))
    } else {
        None
    }
}

/// Extract the argument source text from a `Math.log(x)` call expression.
#[allow(clippy::as_conversions)]
fn extract_math_log_arg_text(source: &str, expr: &Expression<'_>) -> Option<String> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };
    let arg = call.arguments.first()?;
    let arg_span = arg.span();
    let text = source.get(arg_span.start as usize..arg_span.end as usize)?;
    (!text.is_empty()).then(|| text.to_owned())
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferModernMathApis)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_log_div_log2() {
        let diags = lint("var x = Math.log(y) / Math.log(2);");
        assert_eq!(
            diags.len(),
            1,
            "Math.log(y) / Math.log(2) should be flagged"
        );
    }

    #[test]
    fn test_flags_log_div_log10() {
        let diags = lint("var x = Math.log(y) / Math.log(10);");
        assert_eq!(
            diags.len(),
            1,
            "Math.log(y) / Math.log(10) should be flagged"
        );
    }

    #[test]
    fn test_flags_pow_half() {
        let diags = lint("var x = Math.pow(y, 0.5);");
        assert_eq!(diags.len(), 1, "Math.pow(y, 0.5) should be flagged");
    }

    #[test]
    fn test_allows_log2() {
        let diags = lint("var x = Math.log2(y);");
        assert!(diags.is_empty(), "Math.log2 should not be flagged");
    }

    #[test]
    fn test_allows_sqrt() {
        let diags = lint("var x = Math.sqrt(y);");
        assert!(diags.is_empty(), "Math.sqrt should not be flagged");
    }
}
