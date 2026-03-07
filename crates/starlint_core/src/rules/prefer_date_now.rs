//! Rule: `prefer-date-now`
//!
//! Prefer `Date.now()` over `new Date().getTime()` and `+new Date()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

/// Flags `new Date().getTime()` and `+new Date()` — prefer `Date.now()`.
#[derive(Debug)]
pub struct PreferDateNow;

/// Check if a node is `new Date()` (with zero arguments).
fn is_new_date_no_args(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::NewExpression(new_expr)) = ctx.node(node_id) else {
        return false;
    };
    let Some(AstNode::IdentifierReference(id)) = ctx.node(new_expr.callee) else {
        return false;
    };
    id.name.as_str() == "Date" && new_expr.arguments.is_empty()
}

impl LintRule for PreferDateNow {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-date-now".to_owned(),
            description: "Prefer `Date.now()` over `new Date().getTime()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::UnaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // new Date().getTime()
            AstNode::CallExpression(call) => {
                let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                    return;
                };
                if member.property.as_str() != "getTime" {
                    return;
                }
                if !call.arguments.is_empty() {
                    return;
                }
                if !is_new_date_no_args(member.object, ctx) {
                    return;
                }

                ctx.report(Diagnostic {
                    rule_name: "prefer-date-now".to_owned(),
                    message: "Use `Date.now()` instead of `new Date().getTime()`".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace with `Date.now()`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Replace with `Date.now()`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: "Date.now()".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
            // +new Date()
            AstNode::UnaryExpression(unary) => {
                if unary.operator != UnaryOperator::UnaryPlus {
                    return;
                }
                if !is_new_date_no_args(unary.argument, ctx) {
                    return;
                }

                ctx.report(Diagnostic {
                    rule_name: "prefer-date-now".to_owned(),
                    message: "Use `Date.now()` instead of `+new Date()`".to_owned(),
                    span: Span::new(unary.span.start, unary.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace with `Date.now()`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Replace with `Date.now()`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(unary.span.start, unary.span.end),
                            replacement: "Date.now()".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferDateNow)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_date_get_time() {
        let diags = lint("const t = new Date().getTime();");
        assert_eq!(diags.len(), 1, "should flag new Date().getTime()");
    }

    #[test]
    fn test_flags_plus_new_date() {
        let diags = lint("const t = +new Date();");
        assert_eq!(diags.len(), 1, "should flag +new Date()");
    }

    #[test]
    fn test_allows_date_now() {
        let diags = lint("const t = Date.now();");
        assert!(diags.is_empty(), "Date.now() should not be flagged");
    }

    #[test]
    fn test_allows_new_date_with_args() {
        let diags = lint("const t = new Date(2024).getTime();");
        assert!(
            diags.is_empty(),
            "new Date(arg).getTime() should not be flagged"
        );
    }
}
