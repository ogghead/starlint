//! Rule: `no-new-native-nonconstructor`
//!
//! Disallow `new` operators with global non-constructor functions.
//! `Symbol` and `BigInt` are not constructors — using `new Symbol()` or
//! `new BigInt()` throws a `TypeError`. Call them directly instead:
//! `Symbol()` and `BigInt()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Global functions that are not constructors.
const NON_CONSTRUCTORS: &[&str] = &["Symbol", "BigInt"];

/// Flags `new Symbol()` and `new BigInt()`.
#[derive(Debug)]
pub struct NoNewNativeNonconstructor;

impl NativeRule for NoNewNativeNonconstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-native-nonconstructor".to_owned(),
            description: "Disallow `new` with global non-constructor functions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let Expression::Identifier(callee) = &new_expr.callee else {
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewNativeNonconstructor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
