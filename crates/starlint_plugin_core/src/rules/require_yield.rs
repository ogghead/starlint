//! Rule: `require-yield`
//!
//! Require generator functions to contain at least one `yield` expression.
//! A generator function with no `yield` is likely a mistake.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags generator functions that contain no `yield` expressions.
#[derive(Debug)]
pub struct RequireYield;

impl LintRule for RequireYield {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-yield".to_owned(),
            description: "Require generator functions to contain yield".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Function(func) = node else {
            return;
        };

        // Only check generator functions
        if !func.is_generator {
            return;
        }

        // Get the body span to check for yield
        let Some(resolved_body) = func.body.and_then(|body_id| ctx.node(body_id)) else {
            return;
        };

        let bspan = resolved_body.span();

        // Walk the statements looking for yield expressions
        let has_yield = source_contains_yield(ctx.source_text(), bspan.start, bspan.end);

        if !has_yield {
            let name = func
                .id
                .and_then(|id| ctx.node(id))
                .and_then(|n| n.as_binding_identifier())
                .map_or("(anonymous)", |bi| bi.name.as_str());
            ctx.report(Diagnostic {
                rule_name: "require-yield".to_owned(),
                message: format!("Generator function '{name}' requires a yield expression"),
                span: Span::new(func.span.start, func.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Quick check: does the source text in the given span contain the `yield` keyword?
/// This is a simple heuristic — it may false-positive on `yield` in strings/comments,
/// but for generator functions this is almost always correct.
fn source_contains_yield(source: &str, start: u32, end: u32) -> bool {
    let start_idx = usize::try_from(start).unwrap_or(usize::MAX);
    let end_idx = usize::try_from(end).unwrap_or(0).min(source.len());
    source
        .get(start_idx..end_idx)
        .is_some_and(|s| s.contains("yield"))
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(RequireYield);

    #[test]
    fn test_flags_empty_generator() {
        let diags = lint("function* foo() {}");
        assert_eq!(diags.len(), 1, "empty generator should be flagged");
    }

    #[test]
    fn test_allows_generator_with_yield() {
        let diags = lint("function* foo() { yield 1; }");
        assert!(
            diags.is_empty(),
            "generator with yield should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_function() {
        let diags = lint("function foo() {}");
        assert!(diags.is_empty(), "regular function should not be flagged");
    }

    #[test]
    fn test_flags_generator_with_only_return() {
        let diags = lint("function* foo() { return 1; }");
        assert_eq!(
            diags.len(),
            1,
            "generator with only return should be flagged"
        );
    }

    #[test]
    fn test_allows_generator_with_yield_star() {
        let diags = lint("function* foo() { yield* bar(); }");
        assert!(
            diags.is_empty(),
            "generator with yield* should not be flagged"
        );
    }
}
