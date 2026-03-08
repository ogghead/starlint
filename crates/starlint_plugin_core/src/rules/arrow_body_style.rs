//! Rule: `arrow-body-style`
//!
//! Enforce consistent arrow function body style. When an arrow function body
//! contains only a single `return` statement, the block body can be replaced
//! with an expression body.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags arrow functions with block bodies that could use expression bodies.
#[derive(Debug)]
pub struct ArrowBodyStyle;

impl LintRule for ArrowBodyStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "arrow-body-style".to_owned(),
            description: "Enforce consistent arrow function body style".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression])
    }

    #[allow(clippy::indexing_slicing, clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ArrowFunctionExpression(arrow) = node else {
            return;
        };

        // Only check block-body arrows (expression == false means block body)
        if arrow.expression {
            return;
        }

        let arrow_span = Span::new(arrow.span.start, arrow.span.end);

        // Resolve the body via ctx.node() and copy needed values to release borrow.
        let Some(AstNode::FunctionBody(body)) = ctx.node(arrow.body) else {
            return;
        };

        if body.statements.len() != 1 {
            return;
        }

        let body_span = Span::new(body.span.start, body.span.end);
        let first_stmt_id = body.statements[0];
        // body borrow is no longer needed after this point

        // Check if the single statement is a return with an argument
        let arg_id = match ctx.node(first_stmt_id) {
            Some(AstNode::ReturnStatement(ret)) => ret.argument,
            _ => None,
        };

        let Some(arg_id) = arg_id else {
            return;
        };

        let (arg_start, arg_end) = match ctx.node(arg_id) {
            Some(n) => {
                let s = n.span();
                (s.start, s.end)
            }
            None => return,
        };

        // Extract the return value source text
        let arg_text = ctx
            .source_text()
            .get(usize::try_from(arg_start).unwrap_or(0)..usize::try_from(arg_end).unwrap_or(0))
            .unwrap_or("")
            .to_owned();

        // If the return value is an object literal, wrap in parens to avoid
        // ambiguity with block body: `() => ({})` not `() => {}`
        let replacement = if arg_text.starts_with('{') {
            format!("({arg_text})")
        } else {
            arg_text
        };

        ctx.report(Diagnostic {
            rule_name: "arrow-body-style".to_owned(),
            message: "Unexpected block statement surrounding arrow body; move the returned value immediately after `=>`".to_owned(),
            span: arrow_span,
            severity: Severity::Warning,
            help: Some("Replace block body with expression body".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Convert to expression body".to_owned(),
                edits: vec![Edit {
                    span: body_span,
                    replacement,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ArrowBodyStyle)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_block_body_with_single_return() {
        let diags = lint("const f = () => { return 1; };");
        assert_eq!(
            diags.len(),
            1,
            "block body with single return should be flagged"
        );
    }

    #[test]
    fn test_allows_expression_body() {
        let diags = lint("const f = () => 1;");
        assert!(diags.is_empty(), "expression body should not be flagged");
    }

    #[test]
    fn test_allows_multiple_statements() {
        let diags = lint("const f = () => { const x = 1; return x; };");
        assert!(
            diags.is_empty(),
            "multiple statements should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_return() {
        let diags = lint("const f = () => { return; };");
        assert!(
            diags.is_empty(),
            "return without argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_block_body_no_return() {
        let diags = lint("const f = () => { console.log('hi'); };");
        assert!(
            diags.is_empty(),
            "block body without return should not be flagged"
        );
    }
}
