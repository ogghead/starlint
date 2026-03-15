//! Rule: `no-object-constructor`
//!
//! Disallow calls to the `Object` constructor without arguments.
//! Use `{}` instead of `new Object()` or `Object()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `new Object()` and `Object()` without arguments.
#[derive(Debug)]
pub struct NoObjectConstructor;

impl LintRule for NoObjectConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-object-constructor".to_owned(),
            description: "Disallow `Object` constructor".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::NewExpression(new_expr) => {
                if matches!(ctx.node(new_expr.callee), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Object")
                    && new_expr.arguments.is_empty()
                {
                    ctx.report(Diagnostic {
                        rule_name: "no-object-constructor".to_owned(),
                        message: "Disallow `Object` constructor — use `{}` instead".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `{}`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Replace with `{}`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(new_expr.span.start, new_expr.span.end),
                                replacement: "{}".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            AstNode::CallExpression(call) => {
                if matches!(ctx.node(call.callee), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Object")
                    && call.arguments.is_empty()
                {
                    ctx.report(Diagnostic {
                        rule_name: "no-object-constructor".to_owned(),
                        message: "Disallow `Object` constructor — use `{}` instead".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: Some("Replace with `{}`".to_owned()),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: "Replace with `{}`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement: "{}".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoObjectConstructor);

    #[test]
    fn test_flags_new_object() {
        let diags = lint("var x = new Object();");
        assert_eq!(diags.len(), 1, "new Object() should be flagged");
    }

    #[test]
    fn test_flags_object_call() {
        let diags = lint("var x = Object();");
        assert_eq!(diags.len(), 1, "Object() should be flagged");
    }

    #[test]
    fn test_allows_object_literal() {
        let diags = lint("var x = {};");
        assert!(diags.is_empty(), "object literal should not be flagged");
    }

    #[test]
    fn test_allows_object_with_args() {
        let diags = lint("var x = Object(value);");
        assert!(diags.is_empty(), "Object() with args should not be flagged");
    }
}
