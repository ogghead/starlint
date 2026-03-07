//! Rule: `no-extra-bind`
//!
//! Disallow unnecessary `.bind()` calls. If a function does not use `this`,
//! calling `.bind()` on it is unnecessary.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags `.bind()` calls on arrow functions (which cannot be rebound).
#[derive(Debug)]
pub struct NoExtraBind;

impl LintRule for NoExtraBind {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-bind".to_owned(),
            description: "Disallow unnecessary `.bind()` calls".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `.bind()` call
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "bind" {
            return;
        }

        // Arrow functions cannot be rebound -- `.bind()` on them is always useless
        if is_arrow_function(member.object, ctx) {
            let (obj_start, obj_end) = match ctx.node(member.object) {
                Some(n) => {
                    let s = n.span();
                    (s.start, s.end)
                }
                None => (0, 0),
            };
            let source = ctx.source_text();
            let obj_text = source
                .get(usize::try_from(obj_start).unwrap_or(0)..usize::try_from(obj_end).unwrap_or(0))
                .unwrap_or("");

            ctx.report(Diagnostic {
                rule_name: "no-extra-bind".to_owned(),
                message: "The `.bind()` call on an arrow function is unnecessary".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Remove `.bind()`".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `.bind()`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement: obj_text.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is an arrow function, unwrapping parenthesized expressions.
fn is_arrow_function(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.node(expr_id) {
        Some(AstNode::ArrowFunctionExpression(_)) => true,
        // No ParenthesizedExpression in starlint_ast, so just check directly
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtraBind)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_bind_on_arrow() {
        let diags = lint("var f = (() => {}).bind(this);");
        assert_eq!(
            diags.len(),
            1,
            ".bind() on arrow function should be flagged"
        );
    }

    #[test]
    fn test_allows_bind_on_function() {
        let diags = lint("var f = function() { return this; }.bind(obj);");
        assert!(
            diags.is_empty(),
            ".bind() on regular function should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("var f = foo();");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
