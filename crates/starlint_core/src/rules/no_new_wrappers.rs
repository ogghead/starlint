//! Rule: `no-new-wrappers`
//!
//! Disallow `new String()`, `new Number()`, `new Boolean()`.
//! Using primitive wrapper constructors creates objects, not primitives.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `new` on primitive wrapper constructors.
#[derive(Debug)]
pub struct NoNewWrappers;

/// Primitive wrapper types that should not be used with `new`.
const WRAPPER_TYPES: &[&str] = &["String", "Number", "Boolean"];

impl LintRule for NoNewWrappers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-wrappers".to_owned(),
            description: "Disallow primitive wrapper constructors".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        let Some(AstNode::IdentifierReference(id)) = ctx.node(new_expr.callee) else {
            return;
        };

        let name = id.name.as_str();
        if !WRAPPER_TYPES.contains(&name) {
            return;
        }

        let expr_span = Span::new(new_expr.span.start, new_expr.span.end);
        // Fix: remove `new ` prefix, keeping `String(x)` etc.
        let callee_span = id.span;
        let without_new = ctx
            .source_text()
            .get(callee_span.start as usize..expr_span.end as usize)
            .unwrap_or("")
            .to_owned();

        ctx.report(Diagnostic {
            rule_name: "no-new-wrappers".to_owned(),
            message: format!("Do not use `new {name}()` \u{2014} use the primitive instead"),
            span: expr_span,
            severity: Severity::Warning,
            help: Some(format!("Remove `new` to call `{name}()` as a function")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove `new` keyword".to_owned(),
                edits: vec![Edit {
                    span: expr_span,
                    replacement: without_new,
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNewWrappers)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_string() {
        let diags = lint("var s = new String('hello');");
        assert_eq!(diags.len(), 1, "new String() should be flagged");
    }

    #[test]
    fn test_flags_new_number() {
        let diags = lint("var n = new Number(42);");
        assert_eq!(diags.len(), 1, "new Number() should be flagged");
    }

    #[test]
    fn test_flags_new_boolean() {
        let diags = lint("var b = new Boolean(true);");
        assert_eq!(diags.len(), 1, "new Boolean() should be flagged");
    }

    #[test]
    fn test_allows_string_function() {
        let diags = lint("var s = String(42);");
        assert!(
            diags.is_empty(),
            "String() without new should not be flagged"
        );
    }
}
