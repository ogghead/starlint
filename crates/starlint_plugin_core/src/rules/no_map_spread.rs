//! Rule: `no-map-spread`
//!
//! Disallow spreading a `Map` in an object literal (`{...new Map()}`).
//! Map entries don't spread into object properties — the result is an empty
//! object, which is almost certainly a bug. Array spread (`[...new Map()]`)
//! is fine because it yields the Map's entries as `[key, value]` pairs.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `{...new Map()}` in object literals.
#[derive(Debug)]
pub struct NoMapSpread;

impl LintRule for NoMapSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-map-spread".to_owned(),
            description: "Disallow spreading a Map in an object literal".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ObjectExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ObjectExpression(obj) = node else {
            return;
        };

        for &prop_id in &*obj.properties {
            let Some(AstNode::SpreadElement(spread)) = ctx.node(prop_id) else {
                continue;
            };

            // Check if the spread argument is `new Map(...)`.
            let spread_span = spread.span;
            let Some(AstNode::NewExpression(new_expr)) = ctx.node(spread.argument) else {
                continue;
            };

            if let Some(AstNode::IdentifierReference(id)) = ctx.node(new_expr.callee) {
                if id.name.as_str() == "Map" {
                    ctx.report(Diagnostic {
                        rule_name: "no-map-spread".to_owned(),
                        message: "Spreading a Map into an object literal produces an empty object — Map entries are not object properties".to_owned(),
                        span: Span::new(spread_span.start, spread_span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMapSpread)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_spread_new_map() {
        let diags = lint("const x = { ...new Map() };");
        assert_eq!(diags.len(), 1, "spreading new Map() should be flagged");
    }

    #[test]
    fn test_flags_spread_new_map_with_args() {
        let diags = lint("const x = { ...new Map([['a', 1]]) };");
        assert_eq!(
            diags.len(),
            1,
            "spreading new Map(...) with arguments should be flagged"
        );
    }

    #[test]
    fn test_allows_spread_plain_object() {
        let diags = lint("const x = { ...obj };");
        assert!(
            diags.is_empty(),
            "spreading a plain object should not be flagged"
        );
    }

    #[test]
    fn test_allows_spread_new_set() {
        let diags = lint("const x = { ...new Set() };");
        assert!(
            diags.is_empty(),
            "spreading new Set() should not be flagged (only Map is checked)"
        );
    }

    #[test]
    fn test_allows_array_spread_map() {
        let diags = lint("const x = [...new Map()];");
        assert!(
            diags.is_empty(),
            "array spread of new Map() should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_object_properties() {
        let diags = lint("const x = { a: 1, b: 2 };");
        assert!(
            diags.is_empty(),
            "normal object properties should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_map_spreads() {
        let diags = lint("const x = { ...new Map(), ...new Map() };");
        assert_eq!(
            diags.len(),
            2,
            "two Map spreads should produce two diagnostics"
        );
    }
}
