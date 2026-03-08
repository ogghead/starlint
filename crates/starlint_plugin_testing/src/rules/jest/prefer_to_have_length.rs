//! Rule: `jest/prefer-to-have-length`
//!
//! Suggest `expect(arr).toHaveLength(n)` over `expect(arr.length).toBe(n)`.
//! The `toHaveLength` matcher provides clearer failure messages that include
//! the actual length.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(arr.length).toBe(n)` patterns.
#[derive(Debug)]
pub struct PreferToHaveLength;

impl LintRule for PreferToHaveLength {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-have-length".to_owned(),
            description:
                "Suggest using `toHaveLength()` instead of checking `.length` with `toBe()`"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    #[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.toBe(...)` or `.toEqual(...)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let method = member.property.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        // Object must be `expect(...)` call
        let Some(AstNode::CallExpression(expect_call)) = ctx.node(member.object) else {
            return;
        };
        let is_expect = ctx.node(expect_call.callee).is_some_and(
            |n| matches!(n, AstNode::IdentifierReference(id) if id.name.as_str() == "expect"),
        );
        if !is_expect {
            return;
        }

        // First arg of expect() must be `something.length`
        let Some(first_arg_id) = expect_call.arguments.first() else {
            return;
        };
        let Some(AstNode::StaticMemberExpression(arg_member)) = ctx.node(*first_arg_id) else {
            return;
        };
        if arg_member.property.as_str() != "length" {
            return;
        }

        // Two edits:
        // 1. Replace `arr.length` inside expect() with just `arr` (the object of the .length member)
        // 2. Replace the matcher name (`toBe`/`toEqual`) with `toHaveLength`
        let source = ctx.source_text();
        let obj_text = ctx.node(arg_member.object).map_or("", |n| {
            let sp = n.span();
            source.get(sp.start as usize..sp.end as usize).unwrap_or("")
        });

        let arg_member_span = arg_member.span;

        // Find the method name in source to get the span for the fix
        let call_source = source
            .get(call.span.start as usize..call.span.end as usize)
            .unwrap_or("");

        let fix = if let Some(method_idx) = call_source.rfind(method) {
            let method_start = call.span.start + method_idx as u32;
            let method_end = method_start + method.len() as u32;
            Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `toHaveLength`".to_owned(),
                edits: vec![
                    Edit {
                        span: Span::new(arg_member_span.start, arg_member_span.end),
                        replacement: obj_text.to_owned(),
                    },
                    Edit {
                        span: Span::new(method_start, method_end),
                        replacement: "toHaveLength".to_owned(),
                    },
                ],
                is_snippet: false,
            })
        } else {
            None
        };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-to-have-length".to_owned(),
            message: "Use `toHaveLength()` instead of asserting on `.length` with `toBe()`"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `expect(arr).toHaveLength(n)`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferToHaveLength)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_length_to_be() {
        let diags = lint("expect(arr.length).toBe(3);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.length).toBe(3)` should be flagged"
        );
    }

    #[test]
    fn test_flags_length_to_equal() {
        let diags = lint("expect(arr.length).toEqual(0);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.length).toEqual(0)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_length() {
        let diags = lint("expect(arr).toHaveLength(3);");
        assert!(diags.is_empty(), "`toHaveLength()` should not be flagged");
    }
}
