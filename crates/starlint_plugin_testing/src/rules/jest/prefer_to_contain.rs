//! Rule: `jest/prefer-to-contain`
//!
//! Suggest `expect(arr).toContain(x)` over `expect(arr.includes(x)).toBe(true)`.
//! The `toContain` matcher provides a clearer failure message showing the
//! array contents and the missing element.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `expect(arr.includes(x)).toBe(true)` patterns.
#[derive(Debug)]
pub struct PreferToContain;

impl LintRule for PreferToContain {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-contain".to_owned(),
            description:
                "Suggest using `toContain()` instead of `expect(arr.includes(x)).toBe(true)`"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Must be `.toBe(true)` or `.toBe(false)` or `.toEqual(true)` / `.toEqual(false)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let method = member.property.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        // Check the argument is a boolean literal
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

        // The argument to `expect()` must be `something.includes(x)`
        let Some(expect_arg_id) = expect_call.arguments.first() else {
            return;
        };
        let Some(AstNode::CallExpression(includes_call)) = ctx.node(*expect_arg_id) else {
            return;
        };
        let Some(AstNode::StaticMemberExpression(includes_member)) = ctx.node(includes_call.callee)
        else {
            return;
        };
        if includes_member.property.as_str() != "includes" {
            return;
        }

        // Try to build fix: `expect(arr.includes(x)).toBe(true)` -> `expect(arr).toContain(x)`
        let source = ctx.source_text();
        let arr_text = ctx
            .node(includes_member.object)
            .map(|n| {
                let sp = n.span();
                source
                    .get(sp.start as usize..sp.end as usize)
                    .unwrap_or("")
                    .to_owned()
            })
            .unwrap_or_default();
        let includes_arg_text = includes_call.arguments.first().and_then(|aid| {
            let n = ctx.node(*aid)?;
            let sp = n.span();
            Some(
                source
                    .get(sp.start as usize..sp.end as usize)
                    .unwrap_or("")
                    .to_owned(),
            )
        });
        let is_negated = matches!(arg_node, AstNode::BooleanLiteral(b) if !b.value);

        let call_span_start = call.span.start;
        let call_span_end = call.span.end;

        let fix = includes_arg_text.map(|val| {
            let matcher = if is_negated {
                "not.toContain"
            } else {
                "toContain"
            };
            let replacement = format!("expect({arr_text}).{matcher}({val})");
            Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(call_span_start, call_span_end),
                    replacement,
                }],
                is_snippet: false,
            }
        });

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-to-contain".to_owned(),
            message: "Use `toContain()` instead of `expect(arr.includes(x)).toBe(true/false)`"
                .to_owned(),
            span: Span::new(call_span_start, call_span_end),
            severity: Severity::Warning,
            help: Some("Replace with `toContain()`".to_owned()),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferToContain)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_includes_to_be_true() {
        let diags = lint("expect(arr.includes(1)).toBe(true);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.includes(1)).toBe(true)` should be flagged"
        );
    }

    #[test]
    fn test_flags_includes_to_be_false() {
        let diags = lint("expect(arr.includes(1)).toBe(false);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.includes(1)).toBe(false)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_contain() {
        let diags = lint("expect(arr).toContain(1);");
        assert!(diags.is_empty(), "`toContain()` should not be flagged");
    }
}
