//! Rule: `promise/catch-or-return`
//!
//! Require `.catch()` or return for promises. Ensures that promise chains
//! either handle errors via `.catch()` or are returned to the caller.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.then()` calls in expression statements without a trailing `.catch()`.
#[derive(Debug)]
pub struct CatchOrReturn;

impl NativeRule for CatchOrReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/catch-or-return".to_owned(),
            description: "Require `.catch()` or return for promises".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExpressionStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // We match on ExpressionStatement to ensure the promise chain is
        // a top-level statement (not returned or assigned).
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };

        let Expression::CallExpression(call) = &stmt.expression else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();

        // If the outermost call is .catch() or .finally(), that's fine
        if method == "catch" || method == "finally" {
            return;
        }

        // If the outermost call is .then(), flag it
        if method == "then" {
            ctx.report_error(
                "promise/catch-or-return",
                "Promise chain must end with `.catch()` or be returned",
                Span::new(stmt.span.start, stmt.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CatchOrReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_then_without_catch() {
        let diags = lint("promise.then(val => val);");
        assert_eq!(diags.len(), 1, "should flag .then() without .catch()");
    }

    #[test]
    fn test_allows_then_with_catch() {
        let diags = lint("promise.then(val => val).catch(err => err);");
        assert!(diags.is_empty(), ".then().catch() should be allowed");
    }

    #[test]
    fn test_allows_catch_only() {
        let diags = lint("promise.catch(err => console.error(err));");
        assert!(diags.is_empty(), ".catch() alone should be allowed");
    }
}
