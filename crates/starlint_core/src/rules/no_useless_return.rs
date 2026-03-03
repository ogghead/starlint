//! Rule: `no-useless-return`
//!
//! Disallow redundant `return` statements at the end of a function body.
//! A bare `return;` at the end of a function is unnecessary.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags redundant `return;` statements at the end of function bodies.
#[derive(Debug)]
pub struct NoUselessReturn;

impl NativeRule for NoUselessReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-return".to_owned(),
            description: "Disallow redundant return statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::FunctionBody])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::FunctionBody(body) = kind else {
            return;
        };

        // Check if the last statement is a bare return (no argument)
        let Some(last) = body.statements.last() else {
            return;
        };

        if let Statement::ReturnStatement(ret) = last {
            if ret.argument.is_none() {
                ctx.report_warning(
                    "no-useless-return",
                    "Unnecessary return statement",
                    Span::new(ret.span.start, ret.span.end),
                );
            }
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_trailing_return() {
        let diags = lint("function foo() { doSomething(); return; }");
        assert_eq!(diags.len(), 1, "trailing bare return should be flagged");
    }

    #[test]
    fn test_allows_return_with_value() {
        let diags = lint("function foo() { return 42; }");
        assert!(diags.is_empty(), "return with value should not be flagged");
    }

    #[test]
    fn test_allows_no_return() {
        let diags = lint("function foo() { doSomething(); }");
        assert!(
            diags.is_empty(),
            "function without return should not be flagged"
        );
    }
}
