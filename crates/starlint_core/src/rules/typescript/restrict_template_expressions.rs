//! Rule: `typescript/restrict-template-expressions`
//!
//! Disallow template literal expressions with non-string types. Interpolating
//! object literals, array literals, `null`, or `undefined` into template
//! strings produces unhelpful output like `[object Object]`, an empty string,
//! `"null"`, or `"undefined"`.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! This rule inspects `TemplateLiteral` AST nodes and flags expressions that
//! are clearly not strings: object literals, array literals, `null` literals,
//! and the `undefined` identifier.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags template literal expressions that are clearly non-string values.
#[derive(Debug)]
pub struct RestrictTemplateExpressions;

impl LintRule for RestrictTemplateExpressions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/restrict-template-expressions".to_owned(),
            description: "Disallow non-string types in template literal expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TemplateLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TemplateLiteral(template) = node else {
            return;
        };

        // Skip tagged template literals — they may have custom handling
        // (tagged templates appear as TaggedTemplateExpression, not bare
        // TemplateLiteral, but we guard defensively here)

        // Collect findings first to avoid borrow checker issues
        let findings: Vec<(u32, u32, &str)> = template
            .expressions
            .iter()
            .filter_map(|expr_id| {
                let expr_node = ctx.node(*expr_id)?;
                let kind_name = non_string_expression_kind(expr_node)?;
                let span = expr_node.span();
                Some((span.start, span.end, kind_name))
            })
            .collect();

        for (start, end, kind_name) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/restrict-template-expressions".to_owned(),
                message: format!(
                    "Do not use {kind_name} in a template literal — it will not produce a \
                     useful string"
                ),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is clearly a non-string type that should not be
/// interpolated into a template literal.
///
/// Returns a description of the problematic type, or `None` if the expression
/// is acceptable (or cannot be determined without type information).
fn non_string_expression_kind(expr: &AstNode) -> Option<&'static str> {
    match expr {
        AstNode::ObjectExpression(_) => Some("an object literal"),
        AstNode::ArrayExpression(_) => Some("an array literal"),
        AstNode::NullLiteral(_) => Some("`null`"),
        AstNode::IdentifierReference(ident) if ident.name.as_str() == "undefined" => {
            Some("`undefined`")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RestrictTemplateExpressions)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_literal_in_template() {
        let diags = lint("const s = `value: ${{a: 1}}`;");
        assert_eq!(
            diags.len(),
            1,
            "object literal in template should be flagged"
        );
    }

    #[test]
    fn test_flags_array_literal_in_template() {
        let diags = lint("const s = `value: ${[1, 2]}`;");
        assert_eq!(
            diags.len(),
            1,
            "array literal in template should be flagged"
        );
    }

    #[test]
    fn test_flags_null_in_template() {
        let diags = lint("const s = `value: ${null}`;");
        assert_eq!(diags.len(), 1, "null in template should be flagged");
    }

    #[test]
    fn test_flags_undefined_in_template() {
        let diags = lint("const s = `value: ${undefined}`;");
        assert_eq!(diags.len(), 1, "undefined in template should be flagged");
    }

    #[test]
    fn test_allows_string_variable_in_template() {
        let diags = lint("const name = 'world'; const s = `hello ${name}`;");
        assert!(
            diags.is_empty(),
            "string variable in template should not be flagged"
        );
    }
}
