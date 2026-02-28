//! Rule: `prefer-string-slice` (unicorn)
//!
//! Prefer `String#slice()` over `String#substr()` and `String#substring()`.
//! `slice()` is more consistent and handles negative indices intuitively,
//! while `substr()` is deprecated and `substring()` swaps arguments silently
//! when the first is greater than the second.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `substr()` and `substring()` usage.
#[derive(Debug)]
pub struct PreferStringSlice;

impl NativeRule for PreferStringSlice {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-slice".to_owned(),
            description: "Prefer `String#slice()` over `substr()` and `substring()`".to_owned(),
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

        let method = member.property.name.as_str();
        match method {
            "substr" | "substring" => {
                ctx.report_warning(
                    "prefer-string-slice",
                    &format!("Prefer `.slice()` over `.{method}()`"),
                    Span::new(call.span.start, call.span.end),
                );
            }
            _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStringSlice)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_substr() {
        let diags = lint("str.substr(1, 3);");
        assert_eq!(diags.len(), 1, "substr should be flagged");
    }

    #[test]
    fn test_flags_substring() {
        let diags = lint("str.substring(1, 3);");
        assert_eq!(diags.len(), 1, "substring should be flagged");
    }

    #[test]
    fn test_allows_slice() {
        let diags = lint("str.slice(1, 3);");
        assert!(diags.is_empty(), "slice should not be flagged");
    }

    #[test]
    fn test_allows_other_methods() {
        let diags = lint("str.indexOf('x');");
        assert!(diags.is_empty(), "other methods should not be flagged");
    }
}
