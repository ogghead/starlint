//! Rule: `no-invalid-regexp`
//!
//! Disallow invalid regular expression strings in `RegExp` constructors.
//! An invalid regex will throw at runtime and is almost always a bug.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new RegExp(...)` calls with invalid regex patterns.
#[derive(Debug)]
pub struct NoInvalidRegexp;

impl NativeRule for NoInvalidRegexp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-invalid-regexp".to_owned(),
            description: "Disallow invalid regular expression strings in RegExp constructors"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Check if it's `new RegExp(...)`
        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        if ident.name != "RegExp" {
            return;
        }

        // Get the first argument (the pattern)
        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        let oxc_ast::ast::Argument::StringLiteral(pattern_lit) = first_arg else {
            return;
        };

        let pattern = pattern_lit.value.as_str();

        // Get flags if present
        let flags = new_expr
            .arguments
            .get(1)
            .and_then(|arg| {
                if let oxc_ast::ast::Argument::StringLiteral(s) = arg {
                    Some(s.value.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");

        // Validate the regex pattern
        if let Some(error) = validate_regex_pattern(pattern, flags) {
            ctx.report_error(
                "no-invalid-regexp",
                &format!("Invalid regular expression: {error}"),
                Span::new(new_expr.span.start, new_expr.span.end),
            );
        }
    }
}

/// Simple validation of regex patterns. Returns an error message if invalid.
fn validate_regex_pattern(pattern: &str, flags: &str) -> Option<String> {
    // Check for invalid flags
    for ch in flags.chars() {
        if !matches!(ch, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y') {
            return Some(format!(
                "Invalid flags supplied to RegExp constructor '{ch}'"
            ));
        }
    }

    // Check for duplicate flags
    let mut seen_flags = [false; 128];
    for ch in flags.bytes() {
        let idx = usize::from(ch);
        if idx < 128 {
            if *seen_flags.get(idx).unwrap_or(&false) {
                return Some(format!(
                    "Duplicate flag '{}' in RegExp constructor",
                    char::from(ch)
                ));
            }
            if let Some(slot) = seen_flags.get_mut(idx) {
                *slot = true;
            }
        }
    }

    // Check for unbalanced parentheses
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut prev_was_escape = false;

    for ch in pattern.chars() {
        if prev_was_escape {
            prev_was_escape = false;
            continue;
        }
        if ch == '\\' {
            prev_was_escape = true;
            continue;
        }
        if bracket_depth > 0 {
            if ch == ']' {
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            continue;
        }
        match ch {
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                if paren_depth < 0 {
                    return Some("Unmatched ')'".to_owned());
                }
            }
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            _ => {}
        }
    }

    if paren_depth != 0 {
        return Some("Unterminated group".to_owned());
    }

    None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInvalidRegexp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_valid_regexp() {
        let diags = lint("new RegExp('abc');");
        assert!(diags.is_empty(), "valid regex should not be flagged");
    }

    #[test]
    fn test_flags_invalid_flags() {
        let diags = lint("new RegExp('abc', 'z');");
        assert_eq!(diags.len(), 1, "invalid flags should be flagged");
    }

    #[test]
    fn test_flags_duplicate_flags() {
        let diags = lint("new RegExp('abc', 'gg');");
        assert_eq!(diags.len(), 1, "duplicate flags should be flagged");
    }

    #[test]
    fn test_flags_unbalanced_parens() {
        let diags = lint("new RegExp('(abc');");
        assert_eq!(diags.len(), 1, "unbalanced parens should be flagged");
    }

    #[test]
    fn test_allows_valid_flags() {
        let diags = lint("new RegExp('abc', 'gi');");
        assert!(diags.is_empty(), "valid flags should not be flagged");
    }
}
