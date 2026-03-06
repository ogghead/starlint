//! Rule: `new-for-builtins`
//!
//! Enforce `new` for built-in constructors (`Map`, `Set`, `Promise`, etc.)
//! and forbid `new` for factory functions (`Symbol`, `BigInt`).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for NewForBuiltins {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "new-for-builtins".to_owned(),
            description: "Enforce `new` for constructors, forbid for factories".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // Missing `new` for constructors: `Map()` instead of `new Map()`
            AstKind::CallExpression(call) => {
                let Expression::Identifier(id) = &call.callee else {
                    return;
                };
                let name = id.name.as_str();
                if !REQUIRE_NEW.contains(&name) {
                    return;
                }

                ctx.report(Diagnostic {
                    rule_name: "new-for-builtins".to_owned(),
                    message: format!("Use `new {name}()` instead of `{name}()`"),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Add `new` before `{name}`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Add `new` before `{name}`"),
                        edits: vec![Edit {
                            span: Span::new(id.span.start, id.span.start),
                            replacement: "new ".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }

            // Forbidden `new` for factories: `new Symbol()` instead of `Symbol()`
            AstKind::NewExpression(new_expr) => {
                let Expression::Identifier(id) = &new_expr.callee else {
                    return;
                };
                let name = id.name.as_str();
                if !FORBID_NEW.contains(&name) {
                    return;
                }

                // Fix: remove `new ` by replacing the full expression span
                // with the source text from callee start to expression end.
                let callee_start = usize::try_from(id.span.start).unwrap_or(0);
                let expr_end = usize::try_from(new_expr.span.end).unwrap_or(0);
                let replacement = ctx
                    .source_text()
                    .get(callee_start..expr_end)
                    .unwrap_or(name)
                    .to_owned();

                ctx.report(Diagnostic {
                    rule_name: "new-for-builtins".to_owned(),
                    message: format!("`{name}` is not a constructor — do not use `new`"),
                    span: Span::new(new_expr.span.start, new_expr.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Remove `new` before `{name}`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Remove `new` before `{name}`"),
                        edits: vec![Edit {
                            span: Span::new(new_expr.span.start, new_expr.span.end),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NewForBuiltins)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
