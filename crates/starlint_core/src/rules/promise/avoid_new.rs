//! Rule: `promise/avoid-new`
//!
//! Forbid creating `new Promise`. Encourages use of utility functions
//! like `Promise.resolve()`, `Promise.reject()`, or async functions
//! instead of the Promise constructor.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Promise(...)` constructor calls.
#[derive(Debug)]
pub struct AvoidNew;

impl NativeRule for AvoidNew {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/avoid-new".to_owned(),
            description: "Forbid creating `new Promise`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        if ident.name.as_str() == "Promise" {
            ctx.report(Diagnostic {
                rule_name: "promise/avoid-new".to_owned(),
                message: "Avoid creating `new Promise` — prefer async functions or `Promise.resolve()`/`Promise.reject()`".to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AvoidNew)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_promise() {
        let diags = lint("const p = new Promise((resolve) => resolve(1));");
        assert_eq!(diags.len(), 1, "should flag new Promise");
    }

    #[test]
    fn test_allows_promise_resolve() {
        let diags = lint("const p = Promise.resolve(1);");
        assert!(diags.is_empty(), "Promise.resolve should be allowed");
    }

    #[test]
    fn test_allows_other_new() {
        let diags = lint("const m = new Map();");
        assert!(diags.is_empty(), "new Map should not be flagged");
    }
}
