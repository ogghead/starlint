//! Rule: `no-useless-backreference` (eslint)
//!
//! Disallow useless backreferences in regular expressions.
//! A backreference to a group that hasn't been matched yet (forward reference)
//! or to a group in a different alternative will always match the empty string.
//!
//! This is a simplified implementation that catches the most common cases:
//! forward references in `new RegExp(...)`.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags useless backreferences in regular expressions.
#[derive(Debug)]
pub struct NoUselessBackreference;

impl NativeRule for NoUselessBackreference {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-backreference".to_owned(),
            description: "Disallow useless backreferences in regular expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::RegExpLiteral(regex) = kind else {
            return;
        };

        let pattern = regex.regex.pattern.text.as_str();
        if let Some(issue) = find_useless_backreference(pattern) {
            ctx.report_error(
                "no-useless-backreference",
                &issue,
                Span::new(regex.span.start, regex.span.end),
            );
        }
    }
}

/// Find useless backreferences in a regex pattern string.
///
/// Detects forward references (backreference appears before its group)
/// and backreferences to non-existent groups.
fn find_useless_backreference(pattern: &str) -> Option<String> {
    let mut group_count: u32 = 0;
    let bytes = pattern.as_bytes();
    let len = bytes.len();
    let mut i: usize = 0;

    // First pass: count total groups
    let mut total_groups: u32 = 0;
    while i < len {
        let Some(&b) = bytes.get(i) else { break };
        match b {
            b'\\' => {
                // Skip escaped character
                i = i.saturating_add(2);
            }
            b'(' if bytes.get(i.saturating_add(1)) != Some(&b'?') => {
                total_groups = total_groups.saturating_add(1);
                i = i.saturating_add(1);
            }
            _ => {
                i = i.saturating_add(1);
            }
        }
    }

    // Second pass: find backreferences
    i = 0;
    while i < len {
        let Some(&b) = bytes.get(i) else { break };
        match b {
            b'\\' => {
                // Check for backreference
                let next_pos = i.saturating_add(1);
                if let Some(&next_b) = bytes.get(next_pos) {
                    if next_b.is_ascii_digit() && next_b != b'0' {
                        // Parse the backreference number
                        let ref_num = u32::from(next_b.saturating_sub(b'0'));
                        if ref_num > total_groups {
                            return Some(format!(
                                "Backreference `\\{ref_num}` references a non-existent group"
                            ));
                        }
                        if ref_num > group_count {
                            return Some(format!(
                                "Backreference `\\{ref_num}` is a forward reference and will always match empty"
                            ));
                        }
                    }
                }
                i = i.saturating_add(2);
            }
            b'(' if bytes.get(i.saturating_add(1)) != Some(&b'?') => {
                group_count = group_count.saturating_add(1);
                i = i.saturating_add(1);
            }
            b'[' => {
                // Skip character class
                i = i.saturating_add(1);
                while i < len {
                    let Some(&cb) = bytes.get(i) else { break };
                    if cb == b']' {
                        break;
                    }
                    if cb == b'\\' {
                        i = i.saturating_add(1);
                    }
                    i = i.saturating_add(1);
                }
                i = i.saturating_add(1);
            }
            _ => {
                i = i.saturating_add(1);
            }
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessBackreference)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_forward_reference() {
        let diags = lint("var re = /\\1(a)/;");
        assert_eq!(diags.len(), 1, "forward backreference should be flagged");
    }

    #[test]
    fn test_flags_nonexistent_group() {
        let diags = lint("var re = /(a)\\2/;");
        assert_eq!(
            diags.len(),
            1,
            "backreference to nonexistent group should be flagged"
        );
    }

    #[test]
    fn test_allows_valid_backreference() {
        let diags = lint("var re = /(a)\\1/;");
        assert!(
            diags.is_empty(),
            "valid backreference should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_backreference() {
        let diags = lint("var re = /(a)(b)/;");
        assert!(
            diags.is_empty(),
            "regex without backreference should not be flagged"
        );
    }
}
