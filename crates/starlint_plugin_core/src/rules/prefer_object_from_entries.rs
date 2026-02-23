//! Rule: `prefer-object-from-entries`
//!
//! Prefer `Object.fromEntries()` over manual object construction via
//! `.reduce()`. When the initial value of a `.reduce()` call is an empty
//! object literal `{}`, it often indicates manual key-value accumulation
//! that could use `Object.fromEntries()` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.reduce()` calls with an empty object literal as initial value.
#[derive(Debug)]
pub struct PreferObjectFromEntries;

impl LintRule for PreferObjectFromEntries {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-object-from-entries".to_owned(),
            description:
                "Prefer `Object.fromEntries()` over `.reduce()` with an empty object initial value"
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

        let is_reduce = ctx.node(call.callee).and_then(|n| {
            if let AstNode::StaticMemberExpression(member) = n {
                if member.property.as_str() == "reduce" {
                    return Some(());
                }
            }
            None
        });
        if is_reduce.is_none() {
            return;
        }

        // `.reduce(callback, initialValue)` must have exactly 2 arguments
        if call.arguments.len() != 2 {
            return;
        }

        // The second argument (initial value) must be an empty object literal `{}`
        let Some(&second_arg_id) = call.arguments.get(1) else {
            return;
        };

        let is_empty_object = matches!(
            ctx.node(second_arg_id),
            Some(AstNode::ObjectExpression(obj)) if obj.properties.is_empty()
        );

        if !is_empty_object {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-object-from-entries".to_owned(),
            message:
                "Consider using `Object.fromEntries()` instead of `.reduce()` to build an object"
                    .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferObjectFromEntries)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_reduce_with_empty_object() {
        let diags = lint("arr.reduce((acc, item) => ({ ...acc, [item.key]: item.value }), {});");
        assert_eq!(
            diags.len(),
            1,
            ".reduce() with empty object initial value should be flagged"
        );
    }

    #[test]
    fn test_flags_reduce_block_body_with_empty_object() {
        let diags =
            lint("arr.reduce((acc, item) => { acc[item.key] = item.value; return acc; }, {});");
        assert_eq!(
            diags.len(),
            1,
            ".reduce() with block body and empty object should be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_with_number_initial() {
        let diags = lint("arr.reduce((sum, n) => sum + n, 0);");
        assert!(
            diags.is_empty(),
            ".reduce() with number initial value should not be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_with_array_initial() {
        let diags = lint("arr.reduce((acc, item) => acc.concat(item), []);");
        assert!(
            diags.is_empty(),
            ".reduce() with array initial value should not be flagged"
        );
    }

    #[test]
    fn test_allows_object_from_entries() {
        let diags = lint("Object.fromEntries(arr);");
        assert!(
            diags.is_empty(),
            "Object.fromEntries() should not be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_with_non_empty_object() {
        let diags =
            lint("arr.reduce((acc, item) => ({ ...acc, [item.key]: item.value }), { x: 1 });");
        assert!(
            diags.is_empty(),
            ".reduce() with non-empty object initial value should not be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_with_one_argument() {
        let diags = lint("arr.reduce((a, b) => a + b);");
        assert!(
            diags.is_empty(),
            ".reduce() with only one argument should not be flagged"
        );
    }
}
