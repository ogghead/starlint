//! Rule: `new-for-builtins`
//!
//! Enforce `new` for built-in constructors (`Map`, `Set`, `Promise`, etc.)
//! and forbid `new` for factory functions (`Symbol`, `BigInt`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Built-in constructors that require `new`.
const REQUIRE_NEW: &[&str] = &[
    "Array",
    "ArrayBuffer",
    "DataView",
    "Date",
    "Error",
    "FinalizationRegistry",
    "Float32Array",
    "Float64Array",
    "Int8Array",
    "Int16Array",
    "Int32Array",
    "Map",
    "Promise",
    "Proxy",
    "RegExp",
    "Set",
    "SharedArrayBuffer",
    "Uint8Array",
    "Uint16Array",
    "Uint32Array",
    "Uint8ClampedArray",
    "WeakMap",
    "WeakRef",
    "WeakSet",
];

/// Factory functions where `new` is forbidden.
const FORBID_NEW: &[&str] = &["Symbol", "BigInt"];

/// Flags missing or forbidden `new` for built-in constructors and factories.
#[derive(Debug)]
pub struct NewForBuiltins;

impl LintRule for NewForBuiltins {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "new-for-builtins".to_owned(),
            description: "Enforce `new` for constructors, forbid for factories".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression, AstNodeType::NewExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // Missing `new` for constructors: `Map()` instead of `new Map()`
            AstNode::CallExpression(call) => {
                let Some(AstNode::IdentifierReference(id)) = ctx.node(call.callee) else {
                    return;
                };
                let name = id.name.as_str();
                if !REQUIRE_NEW.contains(&name) {
                    return;
                }

                let id_span_start = id.span.start;
                let call_span = Span::new(call.span.start, call.span.end);
                let name_owned = name.to_owned();
                ctx.report(Diagnostic {
                    rule_name: "new-for-builtins".to_owned(),
                    message: format!("Use `new {name_owned}()` instead of `{name_owned}()`"),
                    span: call_span,
                    severity: Severity::Error,
                    help: Some(format!("Add `new` before `{name_owned}`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Add `new` before `{name_owned}`"),
                        edits: vec![Edit {
                            span: Span::new(id_span_start, id_span_start),
                            replacement: "new ".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }

            // Forbidden `new` for factories: `new Symbol()` instead of `Symbol()`
            AstNode::NewExpression(new_expr) => {
                let Some(AstNode::IdentifierReference(id)) = ctx.node(new_expr.callee) else {
                    return;
                };
                let name = id.name.as_str();
                if !FORBID_NEW.contains(&name) {
                    return;
                }

                // Fix: remove `new ` by replacing the full expression span
                // with the source text from callee start to expression end.
                let callee_start = id.span.start as usize;
                let expr_end = new_expr.span.end as usize;
                let name_owned = name.to_owned();
                let replacement = ctx
                    .source_text()
                    .get(callee_start..expr_end)
                    .unwrap_or(&name_owned)
                    .to_owned();

                let new_expr_span = Span::new(new_expr.span.start, new_expr.span.end);
                ctx.report(Diagnostic {
                    rule_name: "new-for-builtins".to_owned(),
                    message: format!("`{name_owned}` is not a constructor — do not use `new`"),
                    span: new_expr_span,
                    severity: Severity::Error,
                    help: Some(format!("Remove `new` before `{name_owned}`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Remove `new` before `{name_owned}`"),
                        edits: vec![Edit {
                            span: new_expr_span,
                            replacement,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NewForBuiltins)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_map_without_new() {
        let diags = lint("const m = Map();");
        assert_eq!(diags.len(), 1, "should flag Map()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("new "),
            "fix should insert 'new '"
        );
    }

    #[test]
    fn test_flags_set_without_new() {
        let diags = lint("const s = Set();");
        assert_eq!(diags.len(), 1, "should flag Set()");
    }

    #[test]
    fn test_flags_date_without_new() {
        let diags = lint("const d = Date();");
        assert_eq!(diags.len(), 1, "should flag Date()");
    }

    #[test]
    fn test_flags_promise_without_new() {
        let diags = lint("const p = Promise(fn);");
        assert_eq!(diags.len(), 1, "should flag Promise()");
    }

    #[test]
    fn test_flags_new_symbol() {
        let diags = lint("const s = new Symbol('x');");
        assert_eq!(diags.len(), 1, "should flag new Symbol()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("Symbol('x')"),
            "fix should remove 'new'"
        );
    }

    #[test]
    fn test_flags_new_bigint() {
        let diags = lint("const b = new BigInt(42);");
        assert_eq!(diags.len(), 1, "should flag new BigInt()");
    }

    #[test]
    fn test_allows_new_map() {
        let diags = lint("const m = new Map();");
        assert!(diags.is_empty(), "new Map() should not be flagged");
    }

    #[test]
    fn test_allows_new_set() {
        let diags = lint("const s = new Set();");
        assert!(diags.is_empty(), "new Set() should not be flagged");
    }

    #[test]
    fn test_allows_symbol_factory() {
        let diags = lint("const s = Symbol('x');");
        assert!(diags.is_empty(), "Symbol() should not be flagged");
    }

    #[test]
    fn test_allows_bigint_factory() {
        let diags = lint("const b = BigInt(42);");
        assert!(diags.is_empty(), "BigInt() should not be flagged");
    }

    #[test]
    fn test_ignores_member_expression() {
        let diags = lint("const m = foo.Map();");
        assert!(diags.is_empty(), "member expression should not be flagged");
    }

    #[test]
    fn test_ignores_non_builtin() {
        let diags = lint("const x = MyClass();");
        assert!(diags.is_empty(), "non-builtin should not be flagged");
    }
}
