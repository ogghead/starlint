//! Rule: `eqeqeq` (unified `LintRule` version)
//!
//! Require `===` and `!==` instead of `==` and `!=`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `==` and `!=` operators, suggesting `===` and `!==` instead.
#[derive(Debug)]
pub struct Eqeqeq;

impl LintRule for Eqeqeq {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "eqeqeq".to_owned(),
            description: "Require `===` and `!==`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        let (replacement, label) = match expr.operator {
            BinaryOperator::Equality => ("===", "=="),
            BinaryOperator::Inequality => ("!==", "!="),
            _ => return,
        };

        // Get child spans to narrow the search window.
        let left_end = ctx
            .node(expr.left)
            .map_or(expr.span.start, |n| n.span().end);
        let right_start = ctx
            .node(expr.right)
            .map_or(expr.span.end, |n| n.span().start);

        let op_span = find_operator_span(ctx.source_text(), left_end, right_start, label);

        ctx.report(Diagnostic {
            rule_name: "eqeqeq".to_owned(),
            message: format!("Expected `{replacement}` and instead saw `{label}`"),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Error,
            help: Some(format!("Use `{replacement}` instead of `{label}`")),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace `{label}` with `{replacement}`"),
                edits: vec![Edit {
                    span: op_span,
                    replacement: replacement.to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Find the span of the operator within a binary expression.
fn find_operator_span(source: &str, start: u32, end: u32, operator: &str) -> Span {
    let source_len = u32::try_from(source.len()).unwrap_or(u32::MAX);
    let clamped_start = usize::try_from(start.min(source_len)).unwrap_or(0);
    let clamped_end = usize::try_from(end.min(source_len)).unwrap_or(0);

    if let Some(slice) = source.get(clamped_start..clamped_end) {
        if let Some(offset) = slice.find(operator) {
            let op_start = start.saturating_add(u32::try_from(offset).unwrap_or(0));
            let op_end = op_start.saturating_add(u32::try_from(operator.len()).unwrap_or(0));
            return Span::new(op_start, op_end);
        }
    }

    Span::new(start, end)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::*;
    use crate::ast_converter;
    use crate::lint_rule::LintRule;
    use crate::traversal::{LintDispatchTable, traverse_ast_tree};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        let tree = ast_converter::convert(&parsed.program);
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(Eqeqeq)];
        let table = LintDispatchTable::build_from_indices(&rules, &[0]);
        traverse_ast_tree(
            &tree,
            &rules,
            &table,
            &[],
            source,
            Path::new("test.js"),
            None,
        )
    }

    #[test]
    fn flags_loose_equality() {
        let diags = lint("if (a == b) {}");
        assert_eq!(diags.len(), 1);
        assert!(diags.first().is_some_and(|d| d.fix.is_some()));
    }

    #[test]
    fn flags_loose_inequality() {
        let diags = lint("if (a != b) {}");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn allows_strict_equality() {
        let diags = lint("if (a === b && c !== d) {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn fix_targets_operator() {
        let source = r#"if ("a == b" == x) {}"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
        if let Some(fix) = diags.first().and_then(|d| d.fix.as_ref()) {
            if let Some(edit) = fix.edits.first() {
                let start = usize::try_from(edit.span.start).unwrap_or(0);
                let end = usize::try_from(edit.span.end).unwrap_or(0);
                let fixed_slice = source.get(start..end).unwrap_or("");
                assert_eq!(fixed_slice, "==");
            }
        }
    }
}
