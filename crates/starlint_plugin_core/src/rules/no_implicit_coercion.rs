//! Rule: `no-implicit-coercion`
//!
//! Disallow shorthand type conversions. Implicit coercions like `!!x`
//! (to boolean), `+x` (to number), or `"" + x` (to string) are less
//! readable than explicit calls like `Boolean(x)`, `Number(x)`, or
//! `String(x)`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags implicit type coercions.
#[derive(Debug)]
pub struct NoImplicitCoercion;

impl LintRule for NoImplicitCoercion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-implicit-coercion".to_owned(),
            description: "Disallow shorthand type conversions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression, AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // !!x → Boolean(x)
            AstNode::UnaryExpression(outer) if outer.operator == UnaryOperator::LogicalNot => {
                let Some(AstNode::UnaryExpression(inner)) = ctx.node(outer.argument) else {
                    return;
                };
                if inner.operator == UnaryOperator::LogicalNot {
                    let inner_arg_span = ctx.node(inner.argument).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let source = ctx.source_text();
                    let arg_start = usize::try_from(inner_arg_span.start).unwrap_or(0);
                    let arg_end = usize::try_from(inner_arg_span.end).unwrap_or(0);
                    let arg_text = source.get(arg_start..arg_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "no-implicit-coercion".to_owned(),
                        message: "Use `Boolean(x)` instead of `!!x`".to_owned(),
                        span: Span::new(outer.span.start, outer.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `Boolean()`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Replace with `Boolean()`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(outer.span.start, outer.span.end),
                                replacement: format!("Boolean({arg_text})"),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            // +x → Number(x) (unary plus on non-numeric)
            AstNode::UnaryExpression(expr) if expr.operator == UnaryOperator::UnaryPlus => {
                // Only flag if the argument is not a numeric literal
                let arg_node = ctx.node(expr.argument);
                if !arg_node.is_some_and(|n| matches!(n, AstNode::NumericLiteral(_))) {
                    let arg_span = arg_node.map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let source = ctx.source_text();
                    let arg_start = usize::try_from(arg_span.start).unwrap_or(0);
                    let arg_end = usize::try_from(arg_span.end).unwrap_or(0);
                    let arg_text = source.get(arg_start..arg_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "no-implicit-coercion".to_owned(),
                        message: "Use `Number(x)` instead of `+x`".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `Number()`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Replace with `Number()`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: format!("Number({arg_text})"),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            // "" + x → String(x)
            AstNode::BinaryExpression(expr) if expr.operator == BinaryOperator::Addition => {
                let left_is_empty_string = ctx
                    .node(expr.left)
                    .is_some_and(|n| matches!(n, AstNode::StringLiteral(s) if s.value.is_empty()));
                if left_is_empty_string {
                    let right_span = ctx.node(expr.right).map_or(
                        starlint_ast::types::Span::new(0, 0),
                        starlint_ast::AstNode::span,
                    );
                    let source = ctx.source_text();
                    let right_start = usize::try_from(right_span.start).unwrap_or(0);
                    let right_end = usize::try_from(right_span.end).unwrap_or(0);
                    let right_text = source.get(right_start..right_end).unwrap_or("x");

                    ctx.report(Diagnostic {
                        rule_name: "no-implicit-coercion".to_owned(),
                        message: "Use `String(x)` instead of `\"\" + x`".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `String()`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Replace with `String()`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: format!("String({right_text})"),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoImplicitCoercion)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_double_negation() {
        let diags = lint("var b = !!x;");
        assert_eq!(diags.len(), 1, "!!x should be flagged");
    }

    #[test]
    fn test_flags_unary_plus() {
        let diags = lint("var n = +x;");
        assert_eq!(diags.len(), 1, "+x should be flagged");
    }

    #[test]
    fn test_flags_empty_string_concat() {
        let diags = lint("var s = '' + x;");
        assert_eq!(diags.len(), 1, "empty string concat should be flagged");
    }

    #[test]
    fn test_allows_boolean_call() {
        let diags = lint("var b = Boolean(x);");
        assert!(diags.is_empty(), "Boolean(x) should not be flagged");
    }

    #[test]
    fn test_allows_number_call() {
        let diags = lint("var n = Number(x);");
        assert!(diags.is_empty(), "Number(x) should not be flagged");
    }

    #[test]
    fn test_allows_unary_plus_on_number() {
        let diags = lint("var n = +5;");
        assert!(
            diags.is_empty(),
            "unary plus on number literal should not be flagged"
        );
    }
}
