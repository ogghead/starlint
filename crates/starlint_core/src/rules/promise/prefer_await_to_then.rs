//! Rule: `promise/prefer-await-to-then`
//!
//! Prefer `async`/`await` over `.then()` chains. Modern async syntax
//! is generally more readable and easier to debug.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.then()` calls, suggesting `async`/`await` instead.
#[derive(Debug)]
pub struct PreferAwaitToThen;

impl NativeRule for PreferAwaitToThen {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/prefer-await-to-then".to_owned(),
            description: "Prefer `async`/`await` over `.then()` chains".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() == "then" {
            ctx.report_warning(
                "promise/prefer-await-to-then",
                "Prefer `async`/`await` over `.then()` chains",
                Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferAwaitToThen)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_then_usage() {
        let diags = lint("promise.then(val => console.log(val));");
        assert_eq!(diags.len(), 1, "should flag .then() usage");
    }

    #[test]
    fn test_allows_await() {
        let diags = lint("async function f() { const val = await promise; }");
        assert!(diags.is_empty(), "await should not be flagged");
    }

    #[test]
    fn test_flags_chained_then() {
        let diags = lint("p.then(a => a).then(b => b);");
        assert_eq!(diags.len(), 2, "should flag both .then() calls");
    }
}
