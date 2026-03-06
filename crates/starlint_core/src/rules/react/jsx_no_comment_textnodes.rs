//! Rule: `react/jsx-no-comment-textnodes`
//!
//! Warn when JSX text contains patterns like `// comment` or `/* comment */`
//! which are rendered as visible text rather than being treated as comments.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-comment-textnodes";

/// Flags JSX text nodes that contain comment-like patterns (`//` or `/* */`)
/// which will be rendered as visible text in the output.
#[derive(Debug)]
pub struct JsxNoCommentTextnodes;

impl NativeRule for JsxNoCommentTextnodes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow comments from being inserted as text nodes".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXText])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXText(text) = kind else {
            return;
        };

        let value = text.value.as_str();

        // Check for line comments: `// ...`
        let has_line_comment = value.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//")
        });

        // Check for block comments: `/* ... */`
        let has_block_comment = value.contains("/*") && value.contains("*/");

        if has_line_comment || has_block_comment {
            // Wrap the text in a JSX expression comment: {/* text */}
            let trimmed = value.trim();
            let comment_text = trimmed
                .trim_start_matches("//")
                .trim_start_matches("/*")
                .trim_end_matches("*/")
                .trim();
            let fix = FixBuilder::new("Wrap in JSX expression comment")
                .replace(
                    Span::new(text.span.start, text.span.end),
                    format!("{{/* {comment_text} */}}"),
                )
                .build();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Comments inside JSX children will be rendered as text. Use JSX expression containers `{/* comment */}` instead".to_owned(),
                span: Span::new(text.span.start, text.span.end),
                severity: Severity::Warning,
                help: Some("Wrap in `{/* ... */}` to make it a proper JSX comment".to_owned()),
                fix,
                labels: vec![],
            });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoCommentTextnodes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_line_comment_in_jsx() {
        let diags = lint("const el = <div>// this is a comment</div>;");
        assert_eq!(diags.len(), 1, "should flag line comment in JSX text");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_block_comment_in_jsx() {
        let diags = lint("const el = <div>/* block comment */</div>;");
        assert_eq!(diags.len(), 1, "should flag block comment in JSX text");
    }

    #[test]
    fn test_allows_normal_text() {
        let diags = lint("const el = <div>Hello world</div>;");
        assert!(diags.is_empty(), "should not flag normal text content");
    }
}
