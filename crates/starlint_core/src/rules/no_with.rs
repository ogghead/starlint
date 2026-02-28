//! Rule: `no-with`
//!
//! Disallow `with` statements. The `with` statement is deprecated in strict
//! mode and creates confusing scope semantics.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `with` statements.
#[derive(Debug)]
pub struct NoWith;

impl NativeRule for NoWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-with".to_owned(),
            description: "Disallow `with` statements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::WithStatement(stmt) = kind else {
            return;
        };

        ctx.report_warning(
            "no-with",
            "Unexpected use of `with` statement",
            Span::new(stmt.span.start, stmt.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoWith)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_with_statement() {
        // Note: with statement only parses in non-strict (sloppy) mode
        let diags = lint("with (obj) { foo; }");
        assert_eq!(diags.len(), 1, "with statement should be flagged");
    }

    #[test]
    fn test_allows_normal_code() {
        let diags = lint("var x = obj.foo;");
        assert!(
            diags.is_empty(),
            "normal property access should not be flagged"
        );
    }
}
