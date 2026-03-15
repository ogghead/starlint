//! Rule: `typescript/no-unnecessary-type-assertion`
//!
//! Flags `x as T` type assertions where the expression is obviously already of
//! type `T`. Without full type inference we detect obvious literal cases:
//! string literal `as string`, number literal `as number`, boolean literal
//! `as boolean`, `null as null`, and `undefined as undefined`.
//!
//! Since `TSAsExpressionNode` has no `type_annotation` field in `starlint_ast`,
//! we use source-text parsing to extract the type keyword after ` as `.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `as T` assertions that are unnecessary because the expression already
/// matches the asserted type.
#[derive(Debug)]
pub struct NoUnnecessaryTypeAssertion;

impl LintRule for NoUnnecessaryTypeAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-type-assertion".to_owned(),
            description: "Disallow type assertions that do not change the type of an expression"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSAsExpression])
    }

    #[allow(clippy::as_conversions)] // u32->usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSAsExpression(expr) = node else {
            return;
        };

        let expr_node = ctx.node(expr.expression);

        // Extract the type annotation from source text.
        // TSAsExpression spans `expr as T`; we find the text after " as ".
        let source = ctx.source_text();
        let full_text = source
            .get(expr.span.start as usize..expr.span.end as usize)
            .unwrap_or("");
        let type_text = full_text.rsplit_once(" as ").map_or("", |(_, t)| t.trim());

        if let Some(description) = is_unnecessary_assertion(expr_node, type_text) {
            // Fix: replace `expr as T` with just `expr`
            let inner_span = expr_node.map(starlint_ast::AstNode::span);
            let inner_text = inner_span
                .and_then(|s| source.get(s.start as usize..s.end as usize))
                .unwrap_or("")
                .to_owned();

            ctx.report(Diagnostic {
                rule_name: "typescript/no-unnecessary-type-assertion".to_owned(),
                message: format!("Unnecessary type assertion: {description}"),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "Remove the `as` assertion \u{2014} replace with `{inner_text}`"
                )),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove unnecessary type assertion".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: inner_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check whether an `as T` assertion is unnecessary because the expression
/// already has the asserted type.
///
/// Returns a human-readable description when the assertion is unnecessary,
/// or `None` when it is (potentially) meaningful.
fn is_unnecessary_assertion(expression: Option<&AstNode>, type_text: &str) -> Option<&'static str> {
    let expr = expression?;
    match (expr, type_text) {
        (AstNode::StringLiteral(_), "string") => Some("string literal is already of type `string`"),
        (AstNode::NumericLiteral(_), "number") => {
            Some("number literal is already of type `number`")
        }
        (AstNode::BooleanLiteral(_), "boolean") => {
            Some("boolean literal is already of type `boolean`")
        }
        (AstNode::NullLiteral(_), "null") => Some("`null` is already of type `null`"),
        (AstNode::IdentifierReference(ident), "undefined") if ident.name == "undefined" => {
            Some("`undefined` is already of type `undefined`")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnnecessaryTypeAssertion, "test.ts");

    #[test]
    fn test_flags_string_literal_as_string() {
        let diags = lint(r#"let x = "hello" as string;"#);
        assert_eq!(
            diags.len(),
            1,
            "string literal asserted as string should be flagged"
        );
    }

    #[test]
    fn test_flags_number_literal_as_number() {
        let diags = lint("let x = 42 as number;");
        assert_eq!(
            diags.len(),
            1,
            "number literal asserted as number should be flagged"
        );
    }

    #[test]
    fn test_flags_boolean_literal_as_boolean() {
        let diags = lint("let x = true as boolean;");
        assert_eq!(
            diags.len(),
            1,
            "boolean literal asserted as boolean should be flagged"
        );
    }

    #[test]
    fn test_flags_null_as_null() {
        let diags = lint("let x = null as null;");
        assert_eq!(diags.len(), 1, "`null as null` should be flagged");
    }

    #[test]
    fn test_allows_meaningful_assertion() {
        let diags = lint("let x = value as string;");
        assert!(
            diags.is_empty(),
            "assertion of non-literal to a type should not be flagged"
        );
    }
}
