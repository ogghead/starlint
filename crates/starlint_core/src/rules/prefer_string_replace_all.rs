//! Rule: `prefer-string-replace-all` (unicorn)
//!
//! Prefer `String#replaceAll()` over `String#replace()` with a global
//! regex. Using `replaceAll` is more readable and clearly communicates
//! the intent to replace all occurrences.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `str.replace(/regex/g, ...)` that could use `str.replaceAll(...)`.
#[derive(Debug)]
pub struct PreferStringReplaceAll;

impl NativeRule for PreferStringReplaceAll {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-replace-all".to_owned(),
            description: "Prefer String#replaceAll() over String#replace() with global regex"
                .to_owned(),
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

        // Check for `something.replace(regex, replacement)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name != "replace" {
            return;
        }

        // Must have at least 2 arguments
        if call.arguments.len() < 2 {
            return;
        }

        // First argument must be a regex with global flag
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let oxc_ast::ast::Argument::RegExpLiteral(regex) = first_arg else {
            return;
        };

        // Check if the regex has the global flag
        let flags = &regex.regex.flags;
        if flags.contains(oxc_ast::ast::RegExpFlags::G) {
            ctx.report_warning(
                "prefer-string-replace-all",
                "Prefer `String#replaceAll()` over `String#replace()` with a global regex",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStringReplaceAll)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_replace_with_global_regex() {
        let diags = lint("str.replace(/foo/g, 'bar');");
        assert_eq!(
            diags.len(),
            1,
            "replace with global regex should be flagged"
        );
    }

    #[test]
    fn test_allows_replace_without_global() {
        let diags = lint("str.replace(/foo/, 'bar');");
        assert!(
            diags.is_empty(),
            "replace without global flag should not be flagged"
        );
    }

    #[test]
    fn test_allows_replace_with_string() {
        let diags = lint("str.replace('foo', 'bar');");
        assert!(
            diags.is_empty(),
            "replace with string should not be flagged"
        );
    }

    #[test]
    fn test_allows_replace_all() {
        let diags = lint("str.replaceAll('foo', 'bar');");
        assert!(diags.is_empty(), "replaceAll should not be flagged");
    }
}
