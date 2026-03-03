//! Rule: `no-obj-calls`
//!
//! Disallow calling global objects as functions. `Math`, `JSON`, `Reflect`,
//! and `Atomics` are not function constructors — calling them like
//! `Math()` or `JSON()` throws a `TypeError` at runtime.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Non-callable global objects.
const NON_CALLABLE_GLOBALS: &[&str] = &["Math", "JSON", "Reflect", "Atomics"];

/// Flags calls to non-callable global objects.
#[derive(Debug)]
pub struct NoObjCalls;

impl NativeRule for NoObjCalls {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-obj-calls".to_owned(),
            description: "Disallow calling global objects as functions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::CallExpression(call) => {
                if let Some(name) = callee_global_name(&call.callee) {
                    if NON_CALLABLE_GLOBALS.contains(&name) {
                        ctx.report_error(
                            "no-obj-calls",
                            &format!("`{name}` is not a function"),
                            Span::new(call.span.start, call.span.end),
                        );
                    }
                }
            }
            AstKind::NewExpression(new_expr) => {
                if let Some(name) = callee_global_name(&new_expr.callee) {
                    if NON_CALLABLE_GLOBALS.contains(&name) {
                        ctx.report_error(
                            "no-obj-calls",
                            &format!("`{name}` is not a constructor"),
                            Span::new(new_expr.span.start, new_expr.span.end),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Extract a simple identifier name from a callee expression.
fn callee_global_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        _ => None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoObjCalls)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_math_call() {
        let diags = lint("var x = Math();");
        assert_eq!(diags.len(), 1, "Math() should be flagged");
    }

    #[test]
    fn test_flags_json_call() {
        let diags = lint("var x = JSON();");
        assert_eq!(diags.len(), 1, "JSON() should be flagged");
    }

    #[test]
    fn test_flags_reflect_call() {
        let diags = lint("var x = Reflect();");
        assert_eq!(diags.len(), 1, "Reflect() should be flagged");
    }

    #[test]
    fn test_flags_atomics_call() {
        let diags = lint("var x = Atomics();");
        assert_eq!(diags.len(), 1, "Atomics() should be flagged");
    }

    #[test]
    fn test_flags_new_math() {
        let diags = lint("var x = new Math();");
        assert_eq!(diags.len(), 1, "new Math() should be flagged");
    }

    #[test]
    fn test_allows_math_method() {
        let diags = lint("var x = Math.floor(1.5);");
        assert!(diags.is_empty(), "Math.floor() should not be flagged");
    }

    #[test]
    fn test_allows_json_method() {
        let diags = lint("var x = JSON.parse('{}');");
        assert!(diags.is_empty(), "JSON.parse() should not be flagged");
    }

    #[test]
    fn test_allows_normal_function_call() {
        let diags = lint("var x = foo();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
