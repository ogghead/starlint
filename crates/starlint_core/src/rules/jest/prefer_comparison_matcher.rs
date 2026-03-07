//! Rule: `jest/prefer-comparison-matcher`
//!
//! Suggest `toBeGreaterThan(x)` / `toBeLessThan(x)` etc. over
//! `expect(a > b).toBe(true)`. The dedicated comparison matchers provide
//! better failure messages showing actual and expected values.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

/// Flags `expect(a > b).toBe(true)` patterns that could use comparison matchers.
#[derive(Debug)]
pub struct PreferComparisonMatcher;

impl LintRule for PreferComparisonMatcher {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-comparison-matcher".to_owned(),
            description: "Suggest using comparison matchers instead of `expect(a > b).toBe(true)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
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

        // First arg of expect() must be a comparison binary expression
        let Some(expect_arg_id) = expect_call.arguments.first() else {
            return;
        };
        let Some(AstNode::BinaryExpression(binary)) = ctx.node(*expect_arg_id) else {
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
            let negated = if is_true { "" } else { ".not" };
            let replacement = format!("expect({left_text}){negated}.{suggestion}({right_text})");
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

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferComparisonMatcher)];
        lint_source(source, "test.js", &rules)
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
