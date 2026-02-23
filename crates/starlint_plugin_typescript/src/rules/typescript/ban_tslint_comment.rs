//! Rule: `typescript/ban-tslint-comment`
//!
//! Disallow `// tslint:` comments. `TSLint` has been deprecated in favor of
//! `ESLint`. Any remaining `tslint:disable` or `tslint:enable` directives
//! should be removed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `tslint:disable` and `tslint:enable` comments in source text.
#[derive(Debug)]
pub struct BanTslintComment;

impl LintRule for BanTslintComment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/ban-tslint-comment".to_owned(),
            description: "Disallow `// tslint:` comments".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let findings = find_tslint_directives(ctx.source_text());

        // Collect all fix data upfront to avoid borrow conflict with ctx
        let fix_data: Vec<_> = findings
            .iter()
            .map(|(directive, start, end)| {
                let source = ctx.source_text();
                let start_usize = usize::try_from(*start).unwrap_or(0);
                let line_start = source
                    .get(..start_usize)
                    .and_then(|s| s.rfind('\n'))
                    .map_or(0, |pos| pos.saturating_add(1));
                let line_end = source
                    .get(start_usize..)
                    .and_then(|s| s.find('\n'))
                    .map_or(source.len(), |pos| {
                        start_usize.saturating_add(pos).saturating_add(1)
                    });
                let delete_start = u32::try_from(line_start).unwrap_or(0);
                let delete_end = u32::try_from(line_end).unwrap_or(delete_start);
                (*directive, *start, *end, delete_start, delete_end)
            })
            .collect();

        for (directive, start, end, delete_start, delete_end) in fix_data {
            ctx.report(Diagnostic {
                rule_name: "typescript/ban-tslint-comment".to_owned(),
                message: format!("Do not use `{directive}` — TSLint has been deprecated"),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some("Remove the tslint comment".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Delete the tslint comment line".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(delete_start, delete_end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Scan source text for `tslint:disable` and `tslint:enable` directives
/// inside comments.
///
/// Returns a list of `(directive, start_offset, end_offset)` tuples for each
/// occurrence that should be flagged.
fn find_tslint_directives(source: &str) -> Vec<(&'static str, u32, u32)> {
    /// `TSLint` directives to detect.
    const TSLINT_DIRECTIVES: &[&str] = &["tslint:disable", "tslint:enable"];

    let mut results = Vec::new();

    for directive in TSLINT_DIRECTIVES {
        let mut search_from: usize = 0;
        while let Some(pos) = source.get(search_from..).and_then(|s| s.find(directive)) {
            let absolute_pos = search_from.saturating_add(pos);
            let after_directive = absolute_pos.saturating_add(directive.len());

            // Only flag when inside a comment context
            if is_inside_comment(source, absolute_pos) {
                let start = u32::try_from(absolute_pos).unwrap_or(0);
                let end = u32::try_from(after_directive).unwrap_or(start);
                results.push((*directive, start, end));
            }

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
    } else if before.contains("//") {
        // No newline — entire prefix is the current line
        return true;
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

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BanTslintComment)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_tslint_disable() {
        let diags = lint("// tslint:disable\nlet x = 1;");
        assert_eq!(diags.len(), 1, "`tslint:disable` comment should be flagged");
    }

    #[test]
    fn test_flags_tslint_enable() {
        let diags = lint("// tslint:enable\nlet x = 1;");
        assert_eq!(diags.len(), 1, "`tslint:enable` comment should be flagged");
    }

    #[test]
    fn test_allows_regular_comment() {
        let diags = lint("// regular comment\nlet x = 1;");
        assert!(diags.is_empty(), "regular comments should not be flagged");
    }
}
