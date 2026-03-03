//! Rule: `no-regex-spaces`
//!
//! Disallow multiple spaces in regular expression literals. Multiple spaces
//! are hard to count and should be replaced with a quantifier, e.g. `/ {3}/`
//! instead of `/   /`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags regular expression literals that contain multiple consecutive spaces.
#[derive(Debug)]
pub struct NoRegexSpaces;

impl NativeRule for NoRegexSpaces {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-regex-spaces".to_owned(),
            description: "Disallow multiple spaces in regular expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::RegExpLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::RegExpLiteral(regex) = kind else {
            return;
        };

        let pattern = regex.regex.pattern.text.as_str();

        // Check for multiple consecutive spaces outside character classes
        if has_multiple_spaces_outside_char_class(pattern) {
            ctx.report_error(
                "no-regex-spaces",
                "Unexpected multiple consecutive spaces in regular expression",
                Span::new(regex.span.start, regex.span.end),
            );
        }
    }
}

/// Check if a regex pattern string has multiple consecutive spaces
/// outside of character classes `[...]`.
fn has_multiple_spaces_outside_char_class(pattern: &str) -> bool {
    let mut in_char_class = false;
    let mut prev_was_space = false;
    let mut prev_was_escape = false;

    for ch in pattern.chars() {
        if prev_was_escape {
            prev_was_escape = false;
            prev_was_space = false;
            continue;
        }

        if ch == '\\' {
            prev_was_escape = true;
            prev_was_space = false;
            continue;
        }

        if ch == '[' && !in_char_class {
            in_char_class = true;
            prev_was_space = false;
            continue;
        }

        if ch == ']' && in_char_class {
            in_char_class = false;
            prev_was_space = false;
            continue;
        }

        if !in_char_class && ch == ' ' {
            if prev_was_space {
                return true;
            }
            prev_was_space = true;
        } else {
            prev_was_space = false;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code with the `NoRegexSpaces` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRegexSpaces)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_multiple_spaces() {
        let diags = lint("var re = /foo  bar/;");
        assert_eq!(diags.len(), 1, "multiple spaces in regex should be flagged");
    }

    #[test]
    fn test_allows_single_space() {
        let diags = lint("var re = /foo bar/;");
        assert!(
            diags.is_empty(),
            "single space in regex should not be flagged"
        );
    }

    #[test]
    fn test_allows_quantifier() {
        let diags = lint("var re = /foo {2}bar/;");
        assert!(
            diags.is_empty(),
            "space with quantifier should not be flagged"
        );
    }

    #[test]
    fn test_allows_spaces_in_char_class() {
        let diags = lint("var re = /[  ]/;");
        assert!(
            diags.is_empty(),
            "spaces inside character class should not be flagged"
        );
    }

    #[test]
    fn test_allows_escaped_space() {
        let diags = lint("var re = /foo\\ \\ bar/;");
        assert!(diags.is_empty(), "escaped spaces should not be flagged");
    }

    #[test]
    fn test_flags_three_spaces() {
        let diags = lint("var re = /foo   bar/;");
        assert_eq!(diags.len(), 1, "three spaces in regex should be flagged");
    }
}
