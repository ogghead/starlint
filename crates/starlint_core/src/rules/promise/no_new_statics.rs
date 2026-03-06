//! Rule: `promise/no-new-statics`
//!
//! Forbid `new Promise.resolve()`, `new Promise.reject()`, `new Promise.all()`,
//! etc. These are static methods and should not be called with `new`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for NoNewStatics {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-new-statics".to_owned(),
            description: "Forbid `new` on Promise static methods".to_owned(),
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

        let Expression::StaticMemberExpression(member) = &new_expr.callee else {
            return;
        };

        let Expression::Identifier(ident) = &member.object else {
            return;
        };

        if ident.name.as_str() != "Promise" {
            return;
        }

        let method = member.property.name.as_str();
        if PROMISE_STATICS.contains(&method) {
            // Remove `new ` prefix: from new_expr start to callee (member expr) start
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove `new` keyword".to_owned(),
                edits: vec![Edit {
                    span: Span::new(new_expr.span.start, member.span.start),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewStatics)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

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
