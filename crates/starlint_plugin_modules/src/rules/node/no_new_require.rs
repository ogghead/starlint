//! Rule: `node/no-new-require`
//!
//! Disallow `new require('...')`. The `require` function is not a
//! constructor. Using `new` with it is almost always a mistake \u{2014}
//! typically the intent is `new (require('module'))()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::fix_utils::source_text_for_span;
use starlint_rule_framework::{FixBuilder, LintContext, LintRule};

/// Flags `new require(...)` expressions.
#[derive(Debug)]
pub struct NoNewRequire;

impl LintRule for NoNewRequire {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-new-require".to_owned(),
            description: "Disallow `new require(...)`".to_owned(),
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

        let is_require = matches!(
            ctx.node(new_expr.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "require"
        );

        if is_require {
            // Fix: new require('x') → new (require('x'))()
            let fix = {
                let source = ctx.source_text();
                let callee_start = ctx.node(new_expr.callee).map_or(0, |n| n.span().start);
                let args_end = new_expr.span.end;
                source_text_for_span(source, Span::new(callee_start, args_end)).and_then(|text| {
                    let replacement = format!("new ({text})()");
                    FixBuilder::new(
                        format!("Replace with `{replacement}`"),
                        FixKind::SuggestionFix,
                    )
                    .replace(
                        Span::new(new_expr.span.start, new_expr.span.end),
                        replacement,
                    )
                    .build()
                })
            };

            ctx.report(Diagnostic {
                rule_name: "node/no-new-require".to_owned(),
                message: "`require` is not a constructor \u{2014} use `new (require('module'))()` to instantiate the export".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: Some("Wrap the require call: `new (require('module'))()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNewRequire);

    #[test]
    fn test_flags_new_require() {
        let diags = lint("var x = new require('x');");
        assert_eq!(diags.len(), 1, "new require() should be flagged");
    }

    #[test]
    fn test_allows_plain_require() {
        let diags = lint("var x = require('x');");
        assert!(diags.is_empty(), "plain require() should not be flagged");
    }

    #[test]
    fn test_allows_new_other_constructor() {
        let diags = lint("var x = new Foo();");
        assert!(diags.is_empty(), "new Foo() should not be flagged");
    }

    #[test]
    fn test_flags_new_require_with_path() {
        let diags = lint("var app = new require('./app');");
        assert_eq!(diags.len(), 1, "new require with path should be flagged");
    }
}
