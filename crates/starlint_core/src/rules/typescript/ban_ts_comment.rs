//! Rule: `typescript/ban-ts-comment`
//!
//! Disallow `@ts-ignore`, `@ts-nocheck`, `@ts-check`, and `@ts-expect-error`
//! comments without a description. These directives suppress `TypeScript`
//! diagnostics and should at minimum include a reason explaining why the
//! suppression is necessary.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Banned `TypeScript` comment directives.
const BANNED_DIRECTIVES: &[&str] = &["@ts-ignore", "@ts-nocheck", "@ts-check", "@ts-expect-error"];

/// Flags banned `TypeScript` comment directives in source text.
#[derive(Debug)]
pub struct BanTsComment;

impl NativeRule for BanTsComment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/ban-ts-comment".to_owned(),
            description: "Disallow `@ts-<directive>` comments without description".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings = find_banned_directives(ctx.source_text());

        for (directive, start, end) in findings {
            // For @ts-ignore, suggest replacing with @ts-expect-error
            let fix = (directive == "@ts-ignore").then(|| Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace `@ts-ignore` with `@ts-expect-error`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(start, end),
                    replacement: "@ts-expect-error".to_owned(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "typescript/ban-ts-comment".to_owned(),
                message: format!("Do not use `{directive}` because it suppresses `TypeScript` diagnostics"),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: (directive == "@ts-ignore").then(|| "Use `@ts-expect-error` instead — it will error when the suppression is no longer needed".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Scan source text for banned `TypeScript` comment directives.
///
/// Returns a list of `(directive, start_offset, end_offset)` tuples for each
/// occurrence that should be flagged. Directives followed by a colon and
/// additional text (a description) are allowed.
fn find_banned_directives(source: &str) -> Vec<(&'static str, u32, u32)> {
    let mut results = Vec::new();

    for directive in BANNED_DIRECTIVES {
        let mut search_from = 0;
        while let Some(pos) = source.get(search_from..).and_then(|s| s.find(directive)) {
            let absolute_pos = search_from.saturating_add(pos);
            let after_directive = absolute_pos.saturating_add(directive.len());

            // Check if this occurrence is inside a comment by scanning backwards
            // for `//` or `/*` on the same line or a preceding block comment opener.
            if !is_inside_comment(source, absolute_pos) {
                search_from = after_directive;
                continue;
            }

            // Check what follows the directive. If a colon followed by non-whitespace
            // text exists, treat it as having a description and allow it.
            if has_description(source, after_directive) {
                search_from = after_directive;
                continue;
            }

            let start = u32::try_from(absolute_pos).unwrap_or(0);
            let end = u32::try_from(after_directive).unwrap_or(start);
            results.push((*directive, start, end));
            search_from = after_directive;
        }
    }

    results
}

/// Check if a position in source text is inside a comment.
///
/// Looks backward from `pos` to find `//` or `/*` indicating the position
/// is within a comment context.
fn is_inside_comment(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");

    // Check for line comment: find the last newline before pos
    if let Some(last_newline) = before.rfind('\n') {
        let line_before = before.get(last_newline..).unwrap_or("");
        if line_before.contains("//") {
            return true;
        }
    } else {
        // No newline — entire prefix is the current line
        if before.contains("//") {
            return true;
        }
    }

    // Check for block comment: find last /* and ensure no */ between it and pos
    if let Some(block_start) = before.rfind("/*") {
        let between = before.get(block_start..).unwrap_or("");
        if !between.contains("*/") {
            return true;
        }
    }

    false
}

/// Check if the text after a directive contains a description.
///
/// A description is indicated by a colon followed by at least one
/// non-whitespace character. For example: `@ts-expect-error: reason here`
fn has_description(source: &str, after_pos: usize) -> bool {
    let rest = source.get(after_pos..).unwrap_or("");

    // Trim leading whitespace before colon
    let trimmed = rest.trim_start();

    // Must start with a colon
    if let Some(after_colon) = trimmed.strip_prefix(':') {
        // Must have non-whitespace text after the colon (on same line)
        let description = after_colon.trim_start();
        if let Some(first_char) = description.chars().next() {
            return first_char != '\n' && first_char != '\r';
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BanTsComment)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_ts_ignore() {
        let diags = lint("// @ts-ignore\nlet x = 1;");
        assert_eq!(diags.len(), 1, "`@ts-ignore` should be flagged");
    }

    #[test]
    fn test_flags_ts_nocheck() {
        let diags = lint("// @ts-nocheck\nlet x = 1;");
        assert_eq!(diags.len(), 1, "`@ts-nocheck` should be flagged");
    }

    #[test]
    fn test_flags_ts_expect_error_block_comment() {
        let diags = lint("/* @ts-expect-error */\nlet x = 1;");
        assert_eq!(
            diags.len(),
            1,
            "`@ts-expect-error` in block comment should be flagged"
        );
    }

    #[test]
    fn test_allows_ts_expect_error_with_description() {
        let diags = lint("// @ts-expect-error: reason here\nlet x = 1;");
        assert!(
            diags.is_empty(),
            "`@ts-expect-error` with description should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_comments() {
        let diags = lint("// This is a normal comment\nlet x = 1;");
        assert!(diags.is_empty(), "normal comments should not be flagged");
    }
}
