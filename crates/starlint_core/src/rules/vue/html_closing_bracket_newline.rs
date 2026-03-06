//! Rule: `vue/html-closing-bracket-newline`
//!
//! Enforce newline before closing bracket of multi-line elements in templates.
//! Scans for multi-line tags where `>` is not on its own line.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/html-closing-bracket-newline";

/// Enforce newline before closing bracket of multi-line elements.
#[derive(Debug)]
pub struct HtmlClosingBracketNewline;

impl NativeRule for HtmlClosingBracketNewline {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce newline before closing bracket of multi-line elements".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Find template-like opening tags that span multiple lines
        // Look for `<` followed by a tag name, then attributes across lines, then `>`
        let mut pos = 0;
        while let Some(open) = source.get(pos..).and_then(|s| s.find('<')) {
            let abs_open = pos.saturating_add(open);
            // Skip closing tags and comments
            let next_char = source
                .get(abs_open.saturating_add(1)..abs_open.saturating_add(2))
                .unwrap_or_default();
            if next_char == "/" || next_char == "!" {
                pos = abs_open.saturating_add(1);
                continue;
            }

            // Find the closing `>`
            let remaining = source.get(abs_open..).unwrap_or_default();
            let Some(close_offset) = remaining.find('>') else {
                break;
            };
            let tag_content = remaining.get(..close_offset).unwrap_or_default();

            // Only check multi-line tags
            if tag_content.contains('\n') {
                // Check what is on the same line as the `>`
                // Get the text just before `>` — the content on the same line
                let before_close = remaining.get(..close_offset).unwrap_or_default();

                // Find the last newline before `>`
                let last_newline = before_close.rfind('\n');
                let same_line_as_close = match last_newline {
                    Some(nl_pos) => before_close
                        .get(nl_pos.saturating_add(1)..)
                        .unwrap_or_default(),
                    None => before_close,
                };

                // If the same line as `>` contains an attribute (`=`), then
                // `>` is on the same line as an attribute — flag it
                if same_line_as_close.contains('=') {
                    let abs_close = abs_open.saturating_add(close_offset);
                    let start = u32::try_from(abs_close).unwrap_or(0);
                    let end = start.saturating_add(1);
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message:
                            "Closing bracket `>` of a multi-line element should be on a new line"
                                .to_owned(),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }

            pos = abs_open.saturating_add(close_offset).saturating_add(1);
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(HtmlClosingBracketNewline)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_single_line_tag() {
        let source = r#"const t = "<div class='foo'>";"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "single-line tag should be allowed");
    }

    #[test]
    fn test_flags_multiline_closing_on_attr_line() {
        let source = "const t = `<div\n  class=\"foo\">`;\n";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "multi-line tag with > on attribute line should be flagged"
        );
    }

    #[test]
    fn test_allows_multiline_closing_on_new_line() {
        let source = "const t = `<div\n  class=\"foo\"\n>`;\n";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "multi-line tag with > on new line should be allowed"
        );
    }
}
