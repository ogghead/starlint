//! Rule: `no-instanceof-builtins` (unicorn)
//!
//! Prefer builtin type-checking methods over `instanceof` for built-in types.
//! `instanceof Array` doesn't work across realms (iframes, workers).
//! Use `Array.isArray()` instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `instanceof` checks on built-in types.
#[derive(Debug)]
pub struct NoInstanceofBuiltins;

/// Built-in types that have better type-checking alternatives.
const BUILTIN_TYPES: &[(&str, &str)] = &[
    ("Array", "Use `Array.isArray()` instead"),
    (
        "ArrayBuffer",
        "Use `ArrayBuffer.isView()` or check constructor",
    ),
    (
        "Error",
        "Use `error instanceof Error` is OK, but consider `cause` chain",
    ),
];

impl LintRule for NoInstanceofBuiltins {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-instanceof-builtins".to_owned(),
            description: "Prefer builtin type-checking methods over instanceof".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(bin) = node else {
            return;
        };

        if !matches!(bin.operator, BinaryOperator::Instanceof) {
            return;
        }

        let Some(AstNode::IdentifierReference(right_id)) = ctx.node(bin.right) else {
            return;
        };

        let name = right_id.name.as_str();
        if let Some((_builtin, suggestion)) = BUILTIN_TYPES.iter().find(|(b, _)| *b == name) {
            // For `x instanceof Array`, offer fix → `Array.isArray(x)`
            #[allow(clippy::as_conversions)]
            let fix = if name == "Array" {
                let source = ctx.source_text();
                let left_span = ctx.node(bin.left).map_or(
                    starlint_ast::types::Span::new(0, 0),
                    starlint_ast::AstNode::span,
                );
                source
                    .get(left_span.start as usize..left_span.end as usize)
                    .map(|left_text| {
                        let replacement = format!("Array.isArray({left_text})");
                        Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `{replacement}`"),
                            edits: vec![Edit {
                                span: Span::new(bin.span.start, bin.span.end),
                                replacement,
                            }],
                            is_snippet: false,
                        }
                    })
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "no-instanceof-builtins".to_owned(),
                message: format!(
                    "Avoid `instanceof {name}` which doesn't work across realms. {suggestion}"
                ),
                span: Span::new(bin.span.start, bin.span.end),
                severity: Severity::Warning,
                help: Some((*suggestion).to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoInstanceofBuiltins);

    #[test]
    fn test_flags_instanceof_array() {
        let diags = lint("if (x instanceof Array) {}");
        assert_eq!(diags.len(), 1, "instanceof Array should be flagged");
    }

    #[test]
    fn test_allows_instanceof_custom() {
        let diags = lint("if (x instanceof MyClass) {}");
        assert!(
            diags.is_empty(),
            "instanceof custom class should not be flagged"
        );
    }

    #[test]
    fn test_allows_array_isarray() {
        let diags = lint("if (Array.isArray(x)) {}");
        assert!(diags.is_empty(), "Array.isArray should not be flagged");
    }
}
