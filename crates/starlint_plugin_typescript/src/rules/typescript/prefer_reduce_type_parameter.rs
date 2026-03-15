//! Rule: `typescript/prefer-reduce-type-parameter`
//!
//! Prefer specifying the generic type argument on `Array.prototype.reduce()`
//! calls instead of using `as` type assertions on the initial value. Writing
//! `.reduce(fn, init as T)` loses type safety; `.reduce<T>(fn, init)` is
//! clearer and preserves the type contract.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `.reduce(fn, init as T)` patterns where the initial value uses a
/// type assertion instead of a generic type parameter.
#[derive(Debug)]
pub struct PreferReduceTypeParameter;

impl LintRule for PreferReduceTypeParameter {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-reduce-type-parameter".to_owned(),
            description:
                "Prefer using a generic type parameter for `reduce` instead of `as` assertions"
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

        // call.callee is a NodeId — resolve it
        let is_reduce = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                return;
            };
            member.property.as_str() == "reduce"
        };

        if !is_reduce {
            return;
        }

        // Check if any argument is a TSAsExpression (i.e. `value as Type`)
        let has_as_assertion = call
            .arguments
            .iter()
            .any(|&arg_id| matches!(ctx.node(arg_id), Some(AstNode::TSAsExpression(_))));

        if has_as_assertion {
            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-reduce-type-parameter".to_owned(),
                message: "Use a generic type parameter on `.reduce<T>()` instead of asserting the initial value with `as`".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferReduceTypeParameter, "test.ts");

    #[test]
    fn test_flags_reduce_with_as_assertion_on_init() {
        let diags =
            lint("const result = arr.reduce((acc, item) => acc, {} as Record<string, number>);");
        assert_eq!(
            diags.len(),
            1,
            "`.reduce()` with `as` on initial value should be flagged"
        );
    }

    #[test]
    fn test_flags_reduce_with_as_assertion_on_any_arg() {
        let diags = lint("const result = arr.reduce((acc, item) => acc, [] as string[]);");
        assert_eq!(
            diags.len(),
            1,
            "`.reduce()` with `as` assertion should be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_with_generic_type() {
        let diags =
            lint("const result = arr.reduce<Record<string, number>>((acc, item) => acc, {});");
        assert!(
            diags.is_empty(),
            "`.reduce()` with generic type parameter should not be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_without_assertion() {
        let diags = lint("const result = arr.reduce((acc, item) => acc + item, 0);");
        assert!(
            diags.is_empty(),
            "`.reduce()` without type assertion should not be flagged"
        );
    }

    #[test]
    fn test_ignores_non_reduce_method() {
        let diags = lint("const result = arr.map((item) => item as string);");
        assert!(
            diags.is_empty(),
            "non-reduce method calls should not be flagged"
        );
    }
}
