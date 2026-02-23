//! Rule: `no-instanceof-array` (unicorn)
//!
//! Disallow `instanceof Array`. Use `Array.isArray()` instead, which works
//! across different realms (iframes, workers).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `x instanceof Array`.
#[derive(Debug)]
pub struct NoInstanceofArray;

impl LintRule for NoInstanceofArray {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-instanceof-array".to_owned(),
            description: "Disallow `instanceof Array` — use `Array.isArray()`".to_owned(),
            category: Category::Correctness,
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

        if expr.operator != BinaryOperator::Instanceof {
            return;
        }

        let is_array = matches!(
            ctx.node(expr.right),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Array"
        );

        if is_array {
            let source = ctx.source_text();
            let left_ast_span = ctx.node(expr.left).map(starlint_ast::AstNode::span);
            let (left_start_u32, left_end_u32) =
                left_ast_span.map_or((0u32, 0u32), |s| (s.start, s.end));
            let left_start = usize::try_from(left_start_u32).unwrap_or(0);
            let left_end = usize::try_from(left_end_u32).unwrap_or(0);
            let left_text = source.get(left_start..left_end).unwrap_or("x");

            ctx.report(Diagnostic {
                rule_name: "no-instanceof-array".to_owned(),
                message: "Use `Array.isArray()` instead of `instanceof Array`".to_owned(),
                span: Span::new(expr.span.start, expr.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `Array.isArray()`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace with `Array.isArray()`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(expr.span.start, expr.span.end),
                        replacement: format!("Array.isArray({left_text})"),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInstanceofArray)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_instanceof_array() {
        let diags = lint("if (x instanceof Array) {}");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_array_is_array() {
        let diags = lint("if (Array.isArray(x)) {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_instanceof_other() {
        let diags = lint("if (x instanceof Map) {}");
        assert!(diags.is_empty());
    }
}
