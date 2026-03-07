//! Rule: `prefer-array-flat-map` (unicorn)
//!
//! Prefer `.flatMap()` over `.map().flat()`. Using `flatMap` is more
//! concise and performs the operation in a single pass.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.map(...).flat()` chains that should use `.flatMap()`.
#[derive(Debug)]
pub struct PreferArrayFlatMap;

impl LintRule for PreferArrayFlatMap {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-array-flat-map".to_owned(),
            description: "Prefer .flatMap() over .map().flat()".to_owned(),
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

        // Check for `.flat()` call
        let Some(AstNode::StaticMemberExpression(flat_member)) = ctx.node(call.callee) else {
            return;
        };

        if flat_member.property != "flat" {
            return;
        }

        // `.flat()` should have 0 args or 1 arg that's the literal `1`
        let is_flat_one = call.arguments.is_empty()
            || (call.arguments.len() == 1
                && call.arguments.first().is_some_and(|arg_id| {
                    matches!(
                        ctx.node(*arg_id),
                        Some(AstNode::NumericLiteral(n)) if (n.value - 1.0).abs() < f64::EPSILON
                    )
                }));

        if !is_flat_one {
            return;
        }

        let flat_member_object = flat_member.object;

        // Check if the object is a `.map(...)` call
        let Some(AstNode::CallExpression(map_call)) = ctx.node(flat_member_object) else {
            return;
        };

        let map_call_span = map_call.span;
        let map_callee_id = map_call.callee;

        let Some(AstNode::StaticMemberExpression(map_member)) = ctx.node(map_callee_id) else {
            return;
        };

        if map_member.property == "map" {
            let call_span = Span::new(call.span.start, call.span.end);
            // Fix: replace `map` with `flatMap` and remove `.flat()` suffix.
            // The `.flat(...)` portion starts right after map_call ends.
            let flat_suffix_span = Span::new(map_call_span.end, call.span.end);
            // For the map property, we need to find it in the source text
            // The map_member span covers the whole static member expression
            // We need to find "map" within it
            let source = ctx.source_text();
            let member_start = usize::try_from(map_member.span.start).unwrap_or(0);
            let member_end = usize::try_from(map_member.span.end).unwrap_or(0);
            let member_text = source.get(member_start..member_end).unwrap_or("");
            // Find the ".map" portion — the property starts after the last dot
            if let Some(dot_pos) = member_text.rfind(".map") {
                let prop_start =
                    u32::try_from(member_start.saturating_add(dot_pos).saturating_add(1))
                        .unwrap_or(0);
                let prop_end = prop_start.saturating_add(3); // "map" is 3 chars
                let map_prop_span = Span::new(prop_start, prop_end);

                ctx.report(Diagnostic {
                    rule_name: "prefer-array-flat-map".to_owned(),
                    message: "Prefer `.flatMap()` over `.map().flat()`".to_owned(),
                    span: call_span,
                    severity: Severity::Warning,
                    help: Some("Use `.flatMap()` instead".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Replace `.map().flat()` with `.flatMap()`".to_owned(),
                        edits: vec![
                            Edit {
                                span: map_prop_span,
                                replacement: "flatMap".to_owned(),
                            },
                            Edit {
                                span: flat_suffix_span,
                                replacement: String::new(),
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferArrayFlatMap)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_map_flat() {
        let diags = lint("arr.map(x => [x]).flat();");
        assert_eq!(diags.len(), 1, ".map().flat() should be flagged");
    }

    #[test]
    fn test_flags_map_flat_one() {
        let diags = lint("arr.map(x => [x]).flat(1);");
        assert_eq!(diags.len(), 1, ".map().flat(1) should be flagged");
    }

    #[test]
    fn test_allows_flat_map() {
        let diags = lint("arr.flatMap(x => [x]);");
        assert!(diags.is_empty(), "flatMap should not be flagged");
    }

    #[test]
    fn test_allows_map_flat_deep() {
        let diags = lint("arr.map(x => [x]).flat(2);");
        assert!(
            diags.is_empty(),
            ".map().flat(2) should not be flagged (deep flat)"
        );
    }

    #[test]
    fn test_allows_flat_alone() {
        let diags = lint("arr.flat();");
        assert!(diags.is_empty(), ".flat() alone should not be flagged");
    }
}
