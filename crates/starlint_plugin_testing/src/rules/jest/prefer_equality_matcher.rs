//! Rule: `jest/prefer-equality-matcher`
//!
//! Suggest `toBe(x)` / `toEqual(x)` over `expect(a === b).toBe(true)`.
//! The dedicated equality matchers produce clearer failure messages with
//! expected vs received values.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(a === b).toBe(true)` patterns.
#[derive(Debug)]
pub struct PreferEqualityMatcher;

impl LintRule for PreferEqualityMatcher {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-equality-matcher".to_owned(),
            description: "Suggest using equality matchers instead of `expect(a === b).toBe(true)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("expect(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.toBe(true)` or `.toBe(false)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let method = member.property.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };
        let Some(arg_node) = ctx.node(*first_arg_id) else {
            return;
        };
        let is_bool = matches!(arg_node, AstNode::BooleanLiteral(_));
        if !is_bool {
            return;
        }
        let is_true = matches!(arg_node, AstNode::BooleanLiteral(b) if b.value);

        // Object must be `expect(...)` call
        let expect_callee_id = member.object;
        let Some(AstNode::CallExpression(expect_call)) = ctx.node(expect_callee_id) else {
            return;
        };
        let is_expect = matches!(
            ctx.node(expect_call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // First arg of expect() must be `a === b` or `a == b` or `a !== b` or `a != b`
        let Some(expect_arg_id) = expect_call.arguments.first() else {
            return;
        };
        let Some(AstNode::BinaryExpression(binary)) = ctx.node(*expect_arg_id) else {
            return;
        };

        let is_equality_op = matches!(
            binary.operator,
            BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::Equality
                | BinaryOperator::Inequality
        );
        if !is_equality_op {
            return;
        }

        // Build fix: extract left/right operands and construct matcher call
        #[allow(clippy::as_conversions)]
        let fix = {
            let source = ctx.source_text();
            let left_span = ctx.node(binary.left).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let right_span = ctx.node(binary.right).map_or(
                starlint_ast::types::Span::EMPTY,
                starlint_ast::AstNode::span,
            );
            let left_text = source
                .get(left_span.start as usize..left_span.end as usize)
                .unwrap_or("");
            let right_text = source
                .get(right_span.start as usize..right_span.end as usize)
                .unwrap_or("");
            // Determine matcher and negation
            let (matcher, negated) = match (binary.operator, is_true) {
                (BinaryOperator::StrictEquality, false)
                | (BinaryOperator::StrictInequality, true) => ("toBe", true),
                (BinaryOperator::Equality, true) | (BinaryOperator::Inequality, false) => {
                    ("toEqual", false)
                }
                (BinaryOperator::Equality, false) | (BinaryOperator::Inequality, true) => {
                    ("toEqual", true)
                }
                _ => ("toBe", false),
            };
            let not_str = if negated { ".not" } else { "" };
            let replacement = format!("expect({left_text}){not_str}.{matcher}({right_text})");
            (!left_text.is_empty() && !right_text.is_empty()).then(|| Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement,
                }],
                is_snippet: false,
            })
        };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-equality-matcher".to_owned(),
            message: "Use `toBe()` or `toEqual()` directly instead of `expect(a === b).toBe(true)`"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferEqualityMatcher);

    #[test]
    fn test_flags_strict_equality() {
        let diags = lint("expect(a === b).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(a === b).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_inequality() {
        let diags = lint("expect(a !== b).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(a !== b).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_allows_direct_to_be() {
        let diags = lint("expect(a).toBe(b);");
        assert!(
            diags.is_empty(),
            "`expect(a).toBe(b)` should not be flagged"
        );
    }
}
