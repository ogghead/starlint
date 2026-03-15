//! Rule: `promise/no-new-statics`
//!
//! Forbid `new Promise.resolve()`, `new Promise.reject()`, `new Promise.all()`,
//! etc. These are static methods and should not be called with `new`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Promise static methods that should never be called with `new`.
const PROMISE_STATICS: &[&str] = &[
    "resolve",
    "reject",
    "all",
    "allSettled",
    "any",
    "race",
    "withResolvers",
];

/// Flags `new Promise.resolve(...)` and similar incorrect usages.
#[derive(Debug)]
pub struct NoNewStatics;

impl LintRule for NoNewStatics {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-new-statics".to_owned(),
            description: "Forbid `new` on Promise static methods".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let info = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(new_expr.callee) else {
                return;
            };
            let is_promise = matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "Promise");
            if !is_promise {
                return;
            }
            (member.property.clone(), member.span)
        };
        let (method, member_span) = info;

        if PROMISE_STATICS.contains(&method.as_str()) {
            // Remove `new ` prefix: from new_expr start to callee (member expr) start
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove `new` keyword".to_owned(),
                edits: vec![Edit {
                    span: Span::new(new_expr.span.start, member_span.start),
                    replacement: String::new(),
                }],
                is_snippet: false,
            });
            ctx.report(Diagnostic {
                rule_name: "promise/no-new-statics".to_owned(),
                message: format!("`Promise.{method}` is a static method — do not use `new`"),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNewStatics);

    #[test]
    fn test_flags_new_promise_resolve() {
        let diags = lint("const p = new Promise.resolve(1);");
        assert_eq!(diags.len(), 1, "should flag new Promise.resolve()");
    }

    #[test]
    fn test_flags_new_promise_all() {
        let diags = lint("const p = new Promise.all([]);");
        assert_eq!(diags.len(), 1, "should flag new Promise.all()");
    }

    #[test]
    fn test_allows_promise_resolve() {
        let diags = lint("const p = Promise.resolve(1);");
        assert!(
            diags.is_empty(),
            "Promise.resolve() without new should be allowed"
        );
    }
}
