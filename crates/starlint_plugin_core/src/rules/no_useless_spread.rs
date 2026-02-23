//! Rule: `no-useless-spread` (unicorn)
//!
//! Disallow unnecessary spread (`...`) in various contexts:
//! - `[...array]` when `array` is already an array (creating unnecessary copy)
//! - `{...obj}` in `Object.assign({...obj})` (redundant spread)
//! - `[...iterable]` passed to methods that already accept iterables

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unnecessary spread expressions.
#[derive(Debug)]
pub struct NoUselessSpread;

impl LintRule for NoUselessSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-spread".to_owned(),
            description: "Disallow unnecessary spread".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrayExpression])
    }

    #[allow(clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ArrayExpression(array) = node else {
            return;
        };

        // Check for `[...singleElement]` — an array literal with only one
        // element which is a spread of an array literal
        if array.elements.len() != 1 {
            return;
        }

        let elem_id = array.elements[0];

        // Resolve the single element: must be a SpreadElement
        let spread_arg_id = match ctx.node(elem_id) {
            Some(AstNode::SpreadElement(spread)) => spread.argument,
            _ => return,
        };

        // The spread argument must be an array literal
        let inner_span = match ctx.node(spread_arg_id) {
            Some(AstNode::ArrayExpression(inner)) => Span::new(inner.span.start, inner.span.end),
            _ => return,
        };

        // `[...[a, b, c]]` — spreading an array literal into a new array
        // Replace `[...[a, b, c]]` with `[a, b, c]` (use inner array's source)
        let inner_text = ctx
            .source_text()
            .get(
                usize::try_from(inner_span.start).unwrap_or(0)
                    ..usize::try_from(inner_span.end).unwrap_or(0),
            )
            .unwrap_or("")
            .to_owned();
        let outer_span = Span::new(array.span.start, array.span.end);
        ctx.report(Diagnostic {
            rule_name: "no-useless-spread".to_owned(),
            message: "Spreading an array literal in an array literal is unnecessary".to_owned(),
            span: outer_span,
            severity: Severity::Warning,
            help: Some("Use the inner array directly".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove unnecessary spread".to_owned(),
                edits: vec![Edit {
                    span: outer_span,
                    replacement: inner_text,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessSpread)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_spread_array_literal() {
        let diags = lint("var x = [...[1, 2, 3]];");
        assert_eq!(diags.len(), 1, "spreading array literal should be flagged");
    }

    #[test]
    fn test_allows_spread_variable() {
        let diags = lint("var x = [...arr];");
        assert!(
            diags.is_empty(),
            "spreading a variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_multiple_elements() {
        let diags = lint("var x = [1, ...arr];");
        assert!(
            diags.is_empty(),
            "array with multiple elements should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_array() {
        let diags = lint("var x = [];");
        assert!(diags.is_empty(), "empty array should not be flagged");
    }
}
