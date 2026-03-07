//! Rule: `no-new-native-nonconstructor`
//!
//! Disallow `new` operators with global non-constructor functions.
//! `Symbol` and `BigInt` are not constructors — using `new Symbol()` or
//! `new BigInt()` throws a `TypeError`. Call them directly instead:
//! `Symbol()` and `BigInt()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Global functions that are not constructors.
const NON_CONSTRUCTORS: &[&str] = &["Symbol", "BigInt"];

/// Flags `new Symbol()` and `new BigInt()`.
#[derive(Debug)]
pub struct NoNewNativeNonconstructor;

impl LintRule for NoNewNativeNonconstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-native-nonconstructor".to_owned(),
            description: "Disallow `new` with global non-constructor functions".to_owned(),
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

        let Some(AstNode::IdentifierReference(callee)) = ctx.node(new_expr.callee) else {
            return;
        };

        let name = callee.name.as_str();
        if NON_CONSTRUCTORS.contains(&name) {
            // Fix: `new Symbol('foo')` → `Symbol('foo')`
            // Remove the `new ` prefix
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let callee_start = callee.span.start as usize;
                let end = new_expr.span.end as usize;
                source.get(callee_start..end).map(|call_text| Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Remove `new` — call `{name}()` directly"),
                    edits: vec![Edit {
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        replacement: call_text.to_owned(),
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: "no-new-native-nonconstructor".to_owned(),
                message: format!("`{name}` is not a constructor"),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Error,
                help: Some(format!("Call `{name}()` without `new`")),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNewNativeNonconstructor)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_symbol() {
        let diags = lint("var x = new Symbol('foo');");
        assert_eq!(diags.len(), 1, "new Symbol() should be flagged");
    }

    #[test]
    fn test_flags_new_bigint() {
        let diags = lint("var x = new BigInt(42);");
        assert_eq!(diags.len(), 1, "new BigInt() should be flagged");
    }

    #[test]
    fn test_allows_symbol_call() {
        let diags = lint("var x = Symbol('foo');");
        assert!(diags.is_empty(), "Symbol() call should not be flagged");
    }

    #[test]
    fn test_allows_bigint_call() {
        let diags = lint("var x = BigInt(42);");
        assert!(diags.is_empty(), "BigInt() call should not be flagged");
    }

    #[test]
    fn test_allows_new_other() {
        let diags = lint("var x = new Map();");
        assert!(diags.is_empty(), "new Map() should not be flagged");
    }

    #[test]
    fn test_allows_new_date() {
        let diags = lint("var x = new Date();");
        assert!(diags.is_empty(), "new Date() should not be flagged");
    }
}
