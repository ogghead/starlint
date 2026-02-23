//! Rule: `no-useless-concat`
//!
//! Disallow unnecessary concatenation of strings or template literals.
//! `"a" + "b"` should just be `"ab"`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags unnecessary concatenation of string literals.
#[derive(Debug)]
pub struct NoUselessConcat;

impl LintRule for NoUselessConcat {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-concat".to_owned(),
            description: "Disallow unnecessary concatenation of strings".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        if expr.operator != BinaryOperator::Addition {
            return;
        }

        // Both sides must be string literals or template literals
        let left_is_string = ctx
            .node(expr.left)
            .is_some_and(|n| matches!(n, AstNode::StringLiteral(_) | AstNode::TemplateLiteral(_)));
        let right_is_string = ctx
            .node(expr.right)
            .is_some_and(|n| matches!(n, AstNode::StringLiteral(_) | AstNode::TemplateLiteral(_)));

        if !left_is_string || !right_is_string {
            return;
        }

        let source = ctx.source_text();
        let left_span = ctx.node(expr.left).map(AstNode::span);
        let right_span = ctx.node(expr.right).map(AstNode::span);

        let fix = left_span.zip(right_span).and_then(|(ls, rs)| {
            let left_raw = source.get(ls.start as usize..ls.end as usize)?;
            let right_raw = source.get(rs.start as usize..rs.end as usize)?;
            if left_raw.len() < 2 || right_raw.len() < 2 {
                return None;
            }
            let left_inner = &left_raw[1..left_raw.len().saturating_sub(1)];
            let right_inner = &right_raw[1..right_raw.len().saturating_sub(1)];
            let quote = &left_raw[..1];
            Some(Fix {
                kind: FixKind::SafeFix,
                message: "Combine into a single string".to_owned(),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
                    replacement: format!("{quote}{left_inner}{right_inner}{quote}"),
                }],
                is_snippet: false,
            })
        });

        ctx.report(Diagnostic {
            rule_name: "no-useless-concat".to_owned(),
            message: "Unnecessary concatenation of two string literals — combine them into one"
                .to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Combine into a single string literal".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessConcat)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_string_concat() {
        let diags = lint("var x = 'a' + 'b';");
        assert_eq!(
            diags.len(),
            1,
            "concatenation of two string literals should be flagged"
        );
    }

    #[test]
    fn test_allows_string_plus_variable() {
        let diags = lint("var x = 'a' + b;");
        assert!(diags.is_empty(), "string + variable should not be flagged");
    }

    #[test]
    fn test_allows_number_addition() {
        let diags = lint("var x = 1 + 2;");
        assert!(diags.is_empty(), "number addition should not be flagged");
    }
}
