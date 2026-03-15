//! Rule: `prefer-template`
//!
//! Suggest using template literals instead of string concatenation.
//! Template literals are more readable when combining strings with
//! variables.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags string concatenation that could use template literals.
#[derive(Debug)]
pub struct PreferTemplate;

impl LintRule for PreferTemplate {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-template".to_owned(),
            description: "Suggest using template literals instead of string concatenation"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        // Check if this is string concatenation (at least one side is a string)
        let left_is_string = is_string_node(expr.left, ctx);
        let right_is_string = is_string_node(expr.right, ctx);

        if !left_is_string && !right_is_string {
            return;
        }

        // Don't flag if both sides are string literals (that's no-useless-concat)
        if left_is_string && right_is_string {
            return;
        }

        // Build fix: convert to template literal
        let source = ctx.source_text();
        let fix = build_template_fix(
            source,
            expr.left,
            expr.right,
            left_is_string,
            right_is_string,
            ctx,
        );

        ctx.report(Diagnostic {
            rule_name: "prefer-template".to_owned(),
            message: "Unexpected string concatenation — prefer template literals".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Use a template literal instead".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Convert to template literal".to_owned(),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement: fix,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check if a node is a string literal or template literal.
fn is_string_node(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(node_id),
        Some(AstNode::StringLiteral(_) | AstNode::TemplateLiteral(_))
    )
}

/// Extract the raw content of a string literal (between quotes) from source,
/// escaping backticks and `${` for template literal context.
fn string_content_for_template(
    source: &str,
    node_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<String> {
    let span = ctx.node(node_id).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let start = usize::try_from(span.start).unwrap_or(0);
    let end = usize::try_from(span.end).unwrap_or(0);
    let raw = source.get(start..end)?;
    // Strip surrounding quotes (single, double, or backtick)
    let inner = raw.get(1..raw.len().saturating_sub(1)).unwrap_or("");
    Some(inner.replace('`', "\\`").replace("${", "\\${"))
}

/// Build the replacement template literal string.
fn build_template_fix(
    source: &str,
    left: NodeId,
    right: NodeId,
    left_is_string: bool,
    right_is_string: bool,
    ctx: &LintContext<'_>,
) -> String {
    let left_span = ctx.node(left).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let right_span = ctx.node(right).map_or(
        starlint_ast::types::Span::EMPTY,
        starlint_ast::AstNode::span,
    );
    let left_start = usize::try_from(left_span.start).unwrap_or(0);
    let left_end = usize::try_from(left_span.end).unwrap_or(0);
    let right_start = usize::try_from(right_span.start).unwrap_or(0);
    let right_end = usize::try_from(right_span.end).unwrap_or(0);

    let left_str = if left_is_string {
        string_content_for_template(source, left, ctx).unwrap_or_default()
    } else {
        let text = source.get(left_start..left_end).unwrap_or("");
        format!("${{{text}}}")
    };

    let right_str = if right_is_string {
        string_content_for_template(source, right, ctx).unwrap_or_default()
    } else {
        let text = source.get(right_start..right_end).unwrap_or("");
        format!("${{{text}}}")
    };

    format!("`{left_str}{right_str}`")
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferTemplate);

    #[test]
    fn test_flags_string_plus_variable() {
        let diags = lint("var x = 'hello ' + name;");
        assert_eq!(diags.len(), 1, "string + variable should be flagged");
    }

    #[test]
    fn test_allows_template_literal() {
        let diags = lint("var x = `hello ${name}`;");
        assert!(diags.is_empty(), "template literal should not be flagged");
    }

    #[test]
    fn test_allows_number_addition() {
        let diags = lint("var x = 1 + 2;");
        assert!(diags.is_empty(), "number addition should not be flagged");
    }

    #[test]
    fn test_allows_string_literal_concat() {
        // This is handled by no-useless-concat
        let diags = lint("var x = 'a' + 'b';");
        assert!(
            diags.is_empty(),
            "string literal concat should not be flagged by prefer-template"
        );
    }
}
