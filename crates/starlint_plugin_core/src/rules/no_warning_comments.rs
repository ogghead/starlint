//! Rule: `no-warning-comments`
//!
//! Disallow specified warning terms in comments. Flags comments containing
//! TODO, FIXME, HACK, etc.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags comments containing warning terms.
#[derive(Debug)]
pub struct NoWarningComments;

/// Default warning terms to flag.
const WARNING_TERMS: &[&str] = &["todo", "fixme", "hack"];

impl LintRule for NoWarningComments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-warning-comments".to_owned(),
            description: "Disallow specified warning terms in comments".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        let lower = source_text.to_ascii_lowercase();
        lower.contains("todo") || lower.contains("fixme") || lower.contains("hack")
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_warning_comments(source);

        for (term, span) in findings {
            ctx.report(Diagnostic {
                rule_name: "no-warning-comments".to_owned(),
                message: format!("Unexpected `{term}` comment"),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }
}

/// Scan source for comments containing warning terms.
/// Returns (term, span) pairs.
fn find_warning_comments(source: &str) -> Vec<(String, Span)> {
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
                check_comment(source, start, end, &mut results);
                pos = end;
                continue;
            } else if next == b'*' {
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
                check_comment(source, start, end, &mut results);
                pos = end;
                continue;
            }
        }

        // Skip string literals
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

/// Check a comment region for warning terms.
fn check_comment(source: &str, start: usize, end: usize, results: &mut Vec<(String, Span)>) {
    let comment = source.get(start..end).unwrap_or("");
    let lower = comment.to_ascii_lowercase();
    for term in WARNING_TERMS {
        if lower.contains(term) {
            if let (Ok(s), Ok(e)) = (u32::try_from(start), u32::try_from(end)) {
                results.push(((*term).to_owned(), Span::new(s, e)));
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoWarningComments)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_todo_comment() {
        let diags = lint("// TODO: fix this");
        assert_eq!(diags.len(), 1, "TODO comment should be flagged");
    }

    #[test]
    fn test_flags_fixme_comment() {
        let diags = lint("/* FIXME: broken */");
        assert_eq!(diags.len(), 1, "FIXME comment should be flagged");
    }

    #[test]
    fn test_allows_normal_comment() {
        let diags = lint("// This is a normal comment");
        assert!(diags.is_empty(), "normal comment should not be flagged");
    }

    #[test]
    fn test_allows_todo_in_string() {
        let diags = lint("var x = 'TODO: fix this';");
        assert!(diags.is_empty(), "TODO in string should not be flagged");
    }
}
