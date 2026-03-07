//! Rule: `vitest/prefer-to-be-object`
//!
//! Suggest `toBeTypeOf('object')` over `typeof` assertions for object checks.
//! Using the Vitest-native `toBeTypeOf` matcher is more readable and provides
//! better error messages than manually comparing `typeof` results.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-to-be-object";

/// Suggest `toBeTypeOf('object')` over manual `typeof` checks.
#[derive(Debug)]
pub struct PreferToBeObject;

impl LintRule for PreferToBeObject {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toBeTypeOf('object')` over `typeof` assertions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        clippy::cast_possible_truncation
    )]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `expect(typeof x).toBe("object")` pattern.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "toBe" {
            return;
        }

        if call.arguments.len() != 1 {
            return;
        }

        // Check if the argument is the string "object".
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let is_object_string = match ctx.node(*first_arg_id) {
            Some(AstNode::StringLiteral(lit)) => lit.value.as_str() == "object",
            _ => false,
        };

        if !is_object_string {
            return;
        }

        // Check if the `expect()` call wraps a `typeof` expression.
        // The member object should be a CallExpression (the `expect(...)` call).
        let Some(AstNode::CallExpression(expect_call)) = ctx.node(member.object) else {
            return;
        };

        let is_expect = matches!(ctx.node(expect_call.callee), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect");

        if !is_expect {
            return;
        }

        // Check if the expect argument is a `typeof` unary expression.
        if let Some(expect_arg_id) = expect_call.arguments.first() {
            if let Some(AstNode::UnaryExpression(unary)) = ctx.node(*expect_arg_id) {
                if unary.operator == UnaryOperator::Typeof {
                    // Two edits:
                    // 1. Replace `typeof x` with `x` inside expect()
                    // 2. Replace `toBe` with `toBeTypeOf`
                    let source = ctx.source_text();
                    let operand_span = ctx.node(unary.argument).map_or(
                        starlint_ast::types::Span::EMPTY,
                        starlint_ast::AstNode::span,
                    );
                    let operand_text = source
                        .get(
                            usize::try_from(operand_span.start).unwrap_or(0)
                                ..usize::try_from(operand_span.end).unwrap_or(0),
                        )
                        .unwrap_or("");
                    let typeof_span = Span::new(unary.span.start, unary.span.end);
                    // member.property is a String, so we can't get its span directly.
                    // Compute the span from the member's span: the property starts after "."
                    // We know the property is "toBe" (4 chars). The property span ends at member.span.end.
                    let prop_len = member.property.len() as u32;
                    let prop_end = member.span.end;
                    let prop_start = prop_end - prop_len;
                    let matcher_span = Span::new(prop_start, prop_end);

                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message:
                            "Prefer `toBeTypeOf('object')` over `expect(typeof x).toBe('object')`"
                                .to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `expect(x).toBeTypeOf('object')`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Replace with `toBeTypeOf`".to_owned(),
                            edits: vec![
                                Edit {
                                    span: typeof_span,
                                    replacement: operand_text.to_owned(),
                                },
                                Edit {
                                    span: matcher_span,
                                    replacement: "toBeTypeOf".to_owned(),
                                },
                            ],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferToBeObject)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_typeof_object_assertion() {
        let source = r#"expect(typeof value).toBe("object");"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`expect(typeof x).toBe('object')` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_be_type_of() {
        let source = r#"expect(value).toBeTypeOf("object");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toBeTypeOf('object')` should not be flagged"
        );
    }

    #[test]
    fn test_allows_to_be_string() {
        let source = r#"expect(typeof value).toBe("string");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`toBe('string')` check should not be flagged by this rule"
        );
    }
}
