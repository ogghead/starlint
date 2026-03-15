//! Rule: `prefer-modern-math-apis` (unicorn)
//!
//! Prefer modern `Math` APIs over legacy patterns. For example:
//! - `Math.log(x) / Math.log(2)` -> `Math.log2(x)`
//! - `Math.log(x) / Math.log(10)` -> `Math.log10(x)`
//! - `Math.pow(x, 0.5)` -> `Math.sqrt(x)` / `Math.cbrt(x)`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags legacy Math patterns that have modern equivalents.
#[derive(Debug)]
pub struct PreferModernMathApis;

impl LintRule for PreferModernMathApis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-modern-math-apis".to_owned(),
            description: "Prefer modern Math APIs".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression, AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    #[allow(clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // Check for Math.log(x) / Math.log(base)
            AstNode::BinaryExpression(bin) => {
                if bin.operator != BinaryOperator::Division {
                    return;
                }

                if is_math_method_call(bin.left, "log", ctx)
                    && is_math_method_call(bin.right, "log", ctx)
                {
                    if let Some((method, suggestion)) =
                        get_log_suggestion_with_method(bin.right, ctx)
                    {
                        // Extract the argument from the numerator Math.log(x)
                        let fix = extract_math_log_arg_text(ctx.source_text(), bin.left, ctx).map(
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
            AstNode::CallExpression(call) => {
                if !is_math_member_callee(call.callee, "pow", ctx) {
                    return;
                }

                if call.arguments.len() != 2 {
                    return;
                }

                let second_arg_id = call.arguments[1];

                if let Some(AstNode::NumericLiteral(num)) = ctx.node(second_arg_id) {
                    #[allow(clippy::float_cmp)]
                    if num.value == 0.5 {
                        // Extract the first argument source text
                        let first_arg_id = call.arguments[0];
                        let first_arg_span = ctx.node(first_arg_id).map_or(
                            starlint_ast::types::Span::EMPTY,
                            starlint_ast::AstNode::span,
                        );
                        let fix = ctx
                            .source_text()
                            .get(first_arg_span.start as usize..first_arg_span.end as usize)
                            .and_then(|arg_text| {
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

/// Check if an expression `NodeId` is `Math.method(...)`.
fn is_math_method_call(expr_id: NodeId, method: &str, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return false;
    };

    is_math_member_callee(call.callee, method, ctx)
}

/// Check if an expression `NodeId` is `Math.method` (as a callee).
fn is_math_member_callee(expr_id: NodeId, method: &str, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(expr_id) else {
        return false;
    };

    let Some(AstNode::IdentifierReference(obj)) = ctx.node(member.object) else {
        return false;
    };

    obj.name == "Math" && member.property == method
}

/// Get the method name and suggestion for `Math.log(x) / Math.log(base)` patterns.
/// Returns `(method_name, full_suggestion)`.
fn get_log_suggestion_with_method(
    divisor_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<(&'static str, &'static str)> {
    let Some(AstNode::CallExpression(call)) = ctx.node(divisor_id) else {
        return None;
    };

    let first_arg_id = *call.arguments.first()?;

    let Some(AstNode::NumericLiteral(num)) = ctx.node(first_arg_id) else {
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
fn extract_math_log_arg_text(
    source: &str,
    expr_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return None;
    };
    let arg_id = *call.arguments.first()?;
    let arg_span = ctx.node(arg_id)?.span();
    let text = source.get(arg_span.start as usize..arg_span.end as usize)?;
    (!text.is_empty()).then(|| text.to_owned())
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferModernMathApis);

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
