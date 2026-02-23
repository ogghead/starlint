//! Rule: `typescript/prefer-as-const`
//!
//! Prefer `as const` over literal type assertion. When a value is asserted to
//! its own literal type (e.g. `"hello" as "hello"` or `1 as 1`), `as const`
//! is clearer and prevents the assertion from drifting out of sync with the
//! value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags literal type assertions that could use `as const` instead.
#[derive(Debug)]
pub struct PreferAsConst;

impl LintRule for PreferAsConst {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-as-const".to_owned(),
            description: "Prefer `as const` over literal type assertion".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSAsExpression, AstNodeType::TSTypeAssertion])
    }

    #[allow(clippy::as_conversions)]
    #[allow(clippy::arithmetic_side_effects)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::TSAsExpression(expr) => {
                // TSAsExpression has expression: NodeId but NO type_annotation field.
                // We need to figure out the type from source text/span.
                // Since TSAsExpressionNode has no type_annotation, we use the span
                // between the expression end and the node end to find the type portion.
                let expr_span = expr.span;
                let expression_id = expr.expression;
                let expression_span = ctx.node(expression_id).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );

                // Get the source text of the type part (after "as ")
                let source = ctx.source_text();
                let expr_text_end = expression_span.end as usize;
                let node_end = expr_span.end as usize;
                let between = source.get(expr_text_end..node_end).unwrap_or("");

                // Find "as " in the between text
                if let Some(as_pos) = between.find("as ") {
                    let type_start = expr_text_end + as_pos + 3; // skip "as "
                    let type_text = source.get(type_start..node_end).unwrap_or("").trim();

                    // Check if the expression is a literal matching the type
                    if is_literal_self_assertion_from_source(expression_id, type_text, ctx) {
                        let type_start_u32 = u32::try_from(type_start).unwrap_or(0);
                        let type_end_u32 = u32::try_from(node_end).unwrap_or(0);
                        ctx.report(Diagnostic {
                            rule_name: "typescript/prefer-as-const".to_owned(),
                            message:
                                "Use `as const` instead of asserting a literal to its own type"
                                    .to_owned(),
                            span: Span::new(expr_span.start, expr_span.end),
                            severity: Severity::Warning,
                            help: Some("Replace with `as const`".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Replace with `as const`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(type_start_u32, type_end_u32),
                                    replacement: "const".to_owned(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            AstNode::TSTypeAssertion(expr) => {
                // TSTypeAssertionNode also has expression: NodeId but NO type_annotation.
                // For angle-bracket syntax `<"hello">"hello"`, replace with `"hello" as const`
                let expr_span = expr.span;
                let expression_id = expr.expression;
                let expression_node_span = ctx.node(expression_id).map_or(
                    starlint_ast::types::Span::EMPTY,
                    starlint_ast::AstNode::span,
                );

                let source = ctx.source_text();
                // Extract the type text between < and >
                let node_text = source
                    .get(expr_span.start as usize..expr_span.end as usize)
                    .unwrap_or("");

                // Look for <Type> at the beginning
                if let (Some(open), Some(close)) = (node_text.find('<'), node_text.find('>')) {
                    let type_text = node_text.get(open + 1..close).unwrap_or("").trim();
                    if is_literal_self_assertion_from_source(expression_id, type_text, ctx) {
                        let expr_text = source
                            .get(
                                expression_node_span.start as usize
                                    ..expression_node_span.end as usize,
                            )
                            .unwrap_or("");
                        let replacement = format!("{expr_text} as const");

                        ctx.report(Diagnostic {
                            rule_name: "typescript/prefer-as-const".to_owned(),
                            message:
                                "Use `as const` instead of asserting a literal to its own type"
                                    .to_owned(),
                            span: Span::new(expr_span.start, expr_span.end),
                            severity: Severity::Warning,
                            help: Some("Replace with `as const`".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Replace with `as const`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(expr_span.start, expr_span.end),
                                    replacement,
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check whether an expression literal matches a type annotation text (source-based comparison).
fn is_literal_self_assertion_from_source(
    expression_id: NodeId,
    type_text: &str,
    ctx: &LintContext<'_>,
) -> bool {
    match ctx.node(expression_id) {
        Some(AstNode::StringLiteral(expr_str)) => {
            // type_text is e.g. `"hello"` and expr value is `hello`
            let unquoted = type_text.trim_matches('"').trim_matches('\'');
            unquoted == expr_str.value
        }
        Some(AstNode::NumericLiteral(expr_num)) => type_text == expr_num.raw,
        Some(AstNode::BooleanLiteral(expr_bool)) => {
            let expected = if expr_bool.value { "true" } else { "false" };
            type_text == expected
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferAsConst)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_string_literal_self_assertion() {
        let diags = lint(r#"let x = "hello" as "hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal asserted to its own type should be flagged"
        );
    }

    #[test]
    fn test_flags_numeric_literal_self_assertion() {
        let diags = lint("let x = 1 as 1;");
        assert_eq!(
            diags.len(),
            1,
            "numeric literal asserted to its own type should be flagged"
        );
    }

    #[test]
    fn test_allows_as_const() {
        let diags = lint(r#"let x = "hello" as const;"#);
        assert!(diags.is_empty(), "`as const` should not be flagged");
    }

    #[test]
    fn test_allows_different_type_assertion() {
        let diags = lint("let x = y as string;");
        assert!(
            diags.is_empty(),
            "assertion to a different type should not be flagged"
        );
    }

    // --- TSTypeAssertion (angle bracket syntax) tests ---

    #[test]
    fn test_flags_angle_bracket_string_assertion() {
        let diags = lint(r#"let x = <"hello">"hello";"#);
        assert_eq!(
            diags.len(),
            1,
            "angle bracket string literal self-assertion should be flagged"
        );
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(
            fix.is_some(),
            "angle bracket string assertion should provide a fix"
        );
    }

    #[test]
    fn test_flags_angle_bracket_numeric_assertion() {
        let diags = lint("let x = <42>42;");
        assert_eq!(
            diags.len(),
            1,
            "angle bracket numeric literal self-assertion should be flagged"
        );
    }

    #[test]
    fn test_flags_angle_bracket_boolean_assertion() {
        let diags = lint("let x = <true>true;");
        assert_eq!(
            diags.len(),
            1,
            "angle bracket boolean literal self-assertion should be flagged"
        );
    }

    #[test]
    fn test_flags_as_boolean_false_assertion() {
        let diags = lint("let x = false as false;");
        assert_eq!(
            diags.len(),
            1,
            "boolean false asserted to its own type should be flagged"
        );
    }

    // --- Valid cases (should NOT be flagged) ---

    #[test]
    fn test_allows_as_const_string() {
        let diags = lint(r#"let x = "hello" as const;"#);
        assert!(
            diags.is_empty(),
            "`as const` on string should not be flagged"
        );
    }

    #[test]
    fn test_allows_angle_bracket_const() {
        let diags = lint(r#"let x = <const>"hello";"#);
        assert!(
            diags.is_empty(),
            "angle bracket const assertion should not be flagged"
        );
    }
}
