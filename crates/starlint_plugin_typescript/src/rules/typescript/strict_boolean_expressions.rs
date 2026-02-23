//! Rule: `typescript/strict-boolean-expressions`
//!
//! Disallow using non-boolean types in boolean contexts. Flags `if` statements
//! whose condition is an obvious non-boolean literal: a string literal, the
//! number `0`, or the empty string `""`. These are almost always mistakes — the
//! developer likely intended a comparison.
//!
//! Simplified syntax-only version — full checking requires type information.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/strict-boolean-expressions";

/// Flags `if` statements whose condition is a non-boolean literal value.
#[derive(Debug)]
pub struct StrictBooleanExpressions;

impl LintRule for StrictBooleanExpressions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow non-boolean types in boolean contexts".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IfStatement])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IfStatement(if_stmt) = node else {
            return;
        };

        let test_node = ctx.node(if_stmt.test);
        let description = test_node.and_then(non_boolean_literal_kind);
        if let Some(description) = description {
            let test_span = test_node.map_or(Span::new(0, 0), |n| {
                let s = n.span();
                Span::new(s.start, s.end)
            });
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Unexpected {description} in boolean context — use an explicit comparison \
                     instead"
                ),
                span: Span::new(test_span.start, test_span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a non-boolean literal that should not appear in a
/// boolean context.
///
/// Returns a human-readable description of the problematic literal, or `None`
/// if the expression is acceptable.
fn non_boolean_literal_kind(expr: &AstNode) -> Option<&'static str> {
    match expr {
        AstNode::StringLiteral(_) => Some("string literal"),
        AstNode::NumericLiteral(lit) if lit.value == 0.0 => Some("numeric literal `0`"),
        AstNode::NullLiteral(_) => Some("`null` literal"),
        AstNode::IdentifierReference(ident) if ident.name.as_str() == "undefined" => {
            Some("`undefined`")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(StrictBooleanExpressions)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_string_literal_in_if() {
        let diags = lint(r#"if ("hello") { console.log("yes"); }"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal in if condition should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_string_in_if() {
        let diags = lint(r#"if ("") { console.log("yes"); }"#);
        assert_eq!(
            diags.len(),
            1,
            "empty string in if condition should be flagged"
        );
    }

    #[test]
    fn test_flags_zero_in_if() {
        let diags = lint("if (0) { console.log('yes'); }");
        assert_eq!(
            diags.len(),
            1,
            "numeric literal 0 in if condition should be flagged"
        );
    }

    #[test]
    fn test_allows_boolean_in_if() {
        let diags = lint("if (true) { console.log('yes'); }");
        assert!(
            diags.is_empty(),
            "boolean literal in if condition should not be flagged"
        );
    }

    #[test]
    fn test_allows_comparison_in_if() {
        let diags = lint("if (x > 0) { console.log('yes'); }");
        assert!(
            diags.is_empty(),
            "comparison expression in if condition should not be flagged"
        );
    }
}
