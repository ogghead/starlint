//! Rule: `no-array-constructor`
//!
//! Disallow `Array` constructors. Use array literal syntax `[]` instead.
//! `new Array(1, 2)` should be `[1, 2]`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `Array()` and `new Array()` with multiple arguments.
#[derive(Debug)]
pub struct NoArrayConstructor;

impl LintRule for NoArrayConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-constructor".to_owned(),
            description: "Disallow `Array` constructor".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (callee_id, arguments, span) = match node {
            AstNode::NewExpression(new_expr) => {
                (new_expr.callee, &new_expr.arguments, new_expr.span)
            }
            AstNode::CallExpression(call) => (call.callee, &call.arguments, call.span),
            _ => return,
        };

        let is_array = matches!(
            ctx.node(callee_id),
            Some(AstNode::IdentifierReference(id)) if id.name == "Array"
        );

        if !is_array || arguments.len() == 1 {
            return;
        }

        let source = ctx.source_text();
        let replacement = build_array_literal(arguments, ctx, source);
        ctx.report(Diagnostic {
            rule_name: "no-array-constructor".to_owned(),
            message: "Use array literal `[]` instead of `Array` constructor".to_owned(),
            span: Span::new(span.start, span.end),
            severity: Severity::Warning,
            help: Some("Replace with array literal".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with array literal".to_owned(),
                edits: vec![Edit {
                    span: Span::new(span.start, span.end),
                    replacement,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Build an array literal string from argument `NodeId`s.
#[allow(clippy::as_conversions)]
fn build_array_literal(args: &[NodeId], ctx: &LintContext<'_>, source: &str) -> String {
    if args.is_empty() {
        return "[]".to_owned();
    }
    let first_span = args.first().and_then(|&id| ctx.node(id)).map(AstNode::span);
    let last_span = args.last().and_then(|&id| ctx.node(id)).map(AstNode::span);
    if let (Some(first), Some(last)) = (first_span, last_span) {
        let args_text = source
            .get(first.start as usize..last.end as usize)
            .unwrap_or("");
        format!("[{args_text}]")
    } else {
        "[]".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoArrayConstructor)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_array_multiple() {
        let diags = lint("var a = new Array(1, 2, 3);");
        assert_eq!(diags.len(), 1, "new Array(1, 2, 3) should be flagged");
    }

    #[test]
    fn test_flags_array_call_empty() {
        let diags = lint("var a = Array();");
        assert_eq!(diags.len(), 1, "Array() empty should be flagged");
    }

    #[test]
    fn test_allows_single_arg() {
        let diags = lint("var a = new Array(5);");
        assert!(
            diags.is_empty(),
            "new Array(5) creates sparse array — should not be flagged"
        );
    }

    #[test]
    fn test_allows_array_literal() {
        let diags = lint("var a = [1, 2, 3];");
        assert!(diags.is_empty(), "array literal should not be flagged");
    }
}
