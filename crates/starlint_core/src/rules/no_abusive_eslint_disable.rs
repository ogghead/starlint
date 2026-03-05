//! Rule: `no-abusive-eslint-disable`
//!
//! Disallow blanket `eslint-disable` comments that do not specify which
//! rule(s) to disable. Using a blanket disable suppresses all warnings and
//! hides legitimate issues — always list the specific rules being suppressed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags blanket `eslint-disable` comments without rule name(s).
#[derive(Debug)]
pub struct NoAbusiveEslintDisable;

/// Disable patterns to check, ordered longest-first so that
/// `eslint-disable-next-line` is tested before `eslint-disable`.
const DISABLE_PATTERNS: &[&str] = &[
    "eslint-disable-next-line",
    "eslint-disable-line",
    "eslint-disable",
];

impl NativeRule for NoAbusiveEslintDisable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-abusive-eslint-disable".to_owned(),
            description: "Disallow blanket `eslint-disable` comments without rule names".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();

        // Early exit: skip files without any eslint-disable comment.
        if !source.contains("eslint-disable") {
            return;
        }

        let findings = find_abusive_disables(source);

        for span in findings {
            ctx.report(Diagnostic {
                rule_name: "no-abusive-eslint-disable".to_owned(),
                message: "Specify the rules to disable — blanket `eslint-disable` hides legitimate issues".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for eslint-disable comments without rule names.
fn find_abusive_disables(source: &str) -> Vec<Span> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos: usize = 0;

    while pos < len {
        let Some(&current) = bytes.get(pos) else {
            break;
        };

        if current == b'/' {
            let Some(&next) = bytes.get(pos.saturating_add(1)) else {
                break;
            };

            if next == b'/' {
                // Line comment: find end
                let start = pos;
                let mut end = pos.saturating_add(2);
                while end < len {
                    if let Some(&ch) = bytes.get(end) {
                        if ch == b'\n' {
                            break;
                        }
                    }
                    end = end.saturating_add(1);
                }
                check_disable_comment(source, start, end, &mut results);
                pos = end;
                continue;
            } else if next == b'*' {
                // Block comment: find closing */
                let start = pos;
                let mut end = pos.saturating_add(2);
                loop {
                    if end.saturating_add(1) >= len {
                        end = len;
                        break;
                    }
                    if let (Some(&c1), Some(&c2)) =
                        (bytes.get(end), bytes.get(end.saturating_add(1)))
                    {
                        if c1 == b'*' && c2 == b'/' {
                            end = end.saturating_add(2);
                            break;
                        }
                    }
                    end = end.saturating_add(1);
                }
                check_disable_comment(source, start, end, &mut results);
                pos = end;
                continue;
            }
        }

        // Skip string literals to avoid false positives
        if current == b'"' || current == b'\'' || current == b'`' {
            let quote = current;
            pos = pos.saturating_add(1);
            while pos < len {
                let Some(&ch) = bytes.get(pos) else {
                    break;
                };
                if ch == b'\\' {
                    pos = pos.saturating_add(2);
                    continue;
                }
                if ch == quote {
                    pos = pos.saturating_add(1);
                    break;
                }
                pos = pos.saturating_add(1);
            }
            continue;
        }

        pos = pos.saturating_add(1);
    }

    results
}

/// Check if a comment contains a blanket eslint-disable pattern.
fn check_disable_comment(source: &str, start: usize, end: usize, results: &mut Vec<Span>) {
    let comment = source.get(start..end).unwrap_or("");

    for pattern in DISABLE_PATTERNS {
        if let Some(pattern_pos) = comment.find(pattern) {
            let after_start = pattern_pos.saturating_add(pattern.len());
            let after = comment.get(after_start..).unwrap_or("");
            let trimmed = after.trim();

            // A blanket disable has nothing meaningful after the keyword —
            // only whitespace, comment-close `*/`, or end of string.
            let is_blanket = trimmed.is_empty() || trimmed == "*/";

            if is_blanket {
                if let (Ok(s), Ok(e)) = (u32::try_from(start), u32::try_from(end)) {
                    results.push(Span::new(s, e));
                }
            }

            // Only check the first matching pattern per comment
            break;
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAbusiveEslintDisable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_blanket_block_disable() {
        let diags = lint("/* eslint-disable */\nvar x = 1;");
        assert_eq!(
            diags.len(),
            1,
            "blanket /* eslint-disable */ should be flagged"
        );
    }

    #[test]
    fn test_flags_blanket_next_line_disable() {
        let diags = lint("// eslint-disable-next-line\nvar x = 1;");
        assert_eq!(
            diags.len(),
            1,
            "blanket // eslint-disable-next-line should be flagged"
        );
    }

    #[test]
    fn test_allows_block_disable_with_rules() {
        let diags = lint("/* eslint-disable no-console */\nvar x = 1;");
        assert!(
            diags.is_empty(),
            "/* eslint-disable no-console */ should not be flagged"
        );
    }

    #[test]
    fn test_allows_line_disable_with_rules() {
        let diags = lint("// eslint-disable-next-line no-alert\nvar x = 1;");
        assert!(
            diags.is_empty(),
            "// eslint-disable-next-line no-alert should not be flagged"
        );
    }

    #[test]
    fn test_allows_disable_in_string() {
        let diags = lint("var x = '/* eslint-disable */';");
        assert!(
            diags.is_empty(),
            "eslint-disable inside a string should not be flagged"
        );
    }

    #[test]
    fn test_flags_blanket_disable_line() {
        let diags = lint("var x = 1; // eslint-disable-line");
        assert_eq!(
            diags.len(),
            1,
            "blanket // eslint-disable-line should be flagged"
        );
    }

    #[test]
    fn test_allows_disable_line_with_rules() {
        let diags = lint("var x = 1; // eslint-disable-line no-unused-vars");
        assert!(
            diags.is_empty(),
            "// eslint-disable-line no-unused-vars should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_comment() {
        let diags = lint("// normal comment\nvar x = 1;");
        assert!(diags.is_empty(), "normal comment should not be flagged");
    }
}
