//! Rule: `require-array-join-separator` (unicorn)
//!
//! Enforce using the separator argument with `Array#join()`.
//! Calling `.join()` without arguments uses `","` as a default separator,
//! which is often not the intent. Require an explicit separator.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Array#join()` calls without an explicit separator argument.
#[derive(Debug)]
pub struct RequireArrayJoinSeparator;

impl NativeRule for RequireArrayJoinSeparator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-array-join-separator".to_owned(),
            description: "Enforce using the separator argument with `Array#join()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "join" {
            return;
        }

        // Flag if no arguments provided
        if call.arguments.is_empty() {
            ctx.report_warning(
                "require-array-join-separator",
                "Missing separator argument in `.join()` — the default `\",\"` may not be intended",
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireArrayJoinSeparator)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_join_without_args() {
        let diags = lint("[1, 2, 3].join();");
        assert_eq!(diags.len(), 1, "join() without args should be flagged");
    }

    #[test]
    fn test_allows_join_with_separator() {
        let diags = lint("[1, 2, 3].join(', ');");
        assert!(
            diags.is_empty(),
            "join with separator should not be flagged"
        );
    }

    #[test]
    fn test_allows_join_with_empty_string() {
        let diags = lint("[1, 2, 3].join('');");
        assert!(
            diags.is_empty(),
            "join with empty string should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_methods() {
        let diags = lint("[1, 2, 3].map(x => x);");
        assert!(diags.is_empty(), "other methods should not be flagged");
    }
}
