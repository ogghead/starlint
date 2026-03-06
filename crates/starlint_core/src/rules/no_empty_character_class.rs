//! Rule: `no-empty-character-class`
//!
//! Disallow empty character classes `[]` in regular expressions. Empty
//! character classes do not match anything and are likely a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags regular expression literals that contain empty character classes.
#[derive(Debug)]
pub struct NoEmptyCharacterClass;

impl NativeRule for NoEmptyCharacterClass {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-character-class".to_owned(),
            description: "Disallow empty character classes in regular expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        if has_empty_character_class(pattern) {
            ctx.report(Diagnostic {
                rule_name: "no-empty-character-class".to_owned(),
                message: "Empty character class `[]` will never match anything".to_owned(),
                span: Span::new(regex.span.start, regex.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a regex pattern contains an empty character class `[]`.
///
/// Handles:
/// - `[^]` is NOT empty (matches any character)
/// - `[\]]` contains an escaped `]`, not empty
/// - `[\\]` is empty (escaped backslash followed by `]`)
fn has_empty_character_class(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes.get(i).copied() == Some(b'\\') {
            // Skip escaped character
            i = i.saturating_add(2);
            continue;
        }

        if bytes.get(i).copied() == Some(b'[') {
            let start = i;
            i = i.saturating_add(1);

            // `[^` is a negated class — skip the `^`
            if i < len && bytes.get(i).copied() == Some(b'^') {
                i = i.saturating_add(1);
            }

            // If we immediately hit `]`, it's an empty character class
            // (or `[^]` which is not empty — it matches any char)
            if i < len && bytes.get(i).copied() == Some(b']') {
                // `[]` is empty, `[^]` is not
                let is_negated = i.saturating_sub(start) == 2;
                if !is_negated {
                    return true;
                }
                i = i.saturating_add(1);
                continue;
            }

            // Walk through the character class looking for the closing `]`
            while i < len {
                if bytes.get(i).copied() == Some(b'\\') {
                    i = i.saturating_add(2);
                    continue;
                }
                if bytes.get(i).copied() == Some(b']') {
                    break;
                }
                i = i.saturating_add(1);
            }
        }

        i = i.saturating_add(1);
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

    /// Helper to lint source code with the `NoEmptyCharacterClass` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyCharacterClass)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_character_class() {
        let diags = lint("var re = /abc[]/;");
        assert_eq!(diags.len(), 1, "empty character class should be flagged");
    }

    #[test]
    fn test_allows_non_empty_character_class() {
        let diags = lint("var re = /abc[a-z]/;");
        assert!(
            diags.is_empty(),
            "non-empty character class should not be flagged"
        );
    }

    #[test]
    fn test_allows_negated_any_match() {
        // [^] in JS matches any character (non-standard but valid)
        let diags = lint("var re = /abc[^]/;");
        assert!(
            diags.is_empty(),
            "negated-any character class should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_character_class() {
        let diags = lint("var re = /abc/;");
        assert!(
            diags.is_empty(),
            "regex without character class should not be flagged"
        );
    }

    #[test]
    fn test_allows_escaped_bracket() {
        let diags = lint("var re = /abc\\[\\]/;");
        assert!(diags.is_empty(), "escaped brackets should not be flagged");
    }
}
