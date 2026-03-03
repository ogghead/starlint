//! Rule: `no-else-return`
//!
//! Disallow `else` blocks after `return` in `if` statements. If the `if`
//! block always returns, the `else` is unnecessary and the code can be
//! flattened.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary `else` blocks after `return`.
#[derive(Debug)]
pub struct NoElseReturn;

impl NativeRule for NoElseReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-else-return".to_owned(),
            description: "Disallow `else` blocks after `return` in `if` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IfStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IfStatement(if_stmt) = kind else {
            return;
        };

        // Must have an else block
        if if_stmt.alternate.is_none() {
            return;
        }

        // The consequent must always return
        if consequent_always_returns(&if_stmt.consequent) {
            ctx.report_warning(
                "no-else-return",
                "Unnecessary `else` after `return` — remove the `else` and outdent its contents",
                Span::new(if_stmt.span.start, if_stmt.span.end),
            );
        }
    }
}

/// Check if a statement always returns.
fn consequent_always_returns(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(_) => true,
        Statement::BlockStatement(block) => block
            .body
            .last()
            .is_some_and(|last| matches!(last, Statement::ReturnStatement(_))),
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoElseReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_else_after_return() {
        let diags = lint("function f(x) { if (x) { return 1; } else { return 2; } }");
        assert_eq!(diags.len(), 1, "else after return should be flagged");
    }

    #[test]
    fn test_allows_no_else() {
        let diags = lint("function f(x) { if (x) { return 1; } return 2; }");
        assert!(diags.is_empty(), "no else should not be flagged");
    }

    #[test]
    fn test_allows_no_return_in_if() {
        let diags = lint("function f(x) { if (x) { foo(); } else { bar(); } }");
        assert!(diags.is_empty(), "if without return should not be flagged");
    }
}
