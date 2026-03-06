//! Rule: `prefer-string-raw`
//!
//! Prefer `String.raw` for template literals that contain backslash escape
//! sequences but no expressions. When a template literal has escapes and
//! nothing else, `String.raw` makes the intent clearer and avoids
//! double-escaping issues.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags template literals with escape sequences that could use `String.raw`.
#[derive(Debug)]
pub struct PreferStringRaw;

impl NativeRule for PreferStringRaw {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-raw".to_owned(),
            description: "Prefer `String.raw` for template literals with backslash escapes"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TemplateLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TemplateLiteral(template) = kind else {
            return;
        };

        // Must have no expressions (pure static template)
        if !template.expressions.is_empty() {
            return;
        }

        // Must have exactly one quasi element
        if template.quasis.len() != 1 {
            return;
        }

        let Some(quasi) = template.quasis.first() else {
            return;
        };

        // If the raw text contains a backslash, there are escape sequences.
        // The `raw` field preserves the original source text (including `\n`,
        // `\t`, etc.) while the `cooked` field contains the processed result.
        // When they differ, escapes are present.
        let raw = quasi.value.raw.as_str();
        if !raw.contains('\\') {
            return;
        }

        // Skip if already inside a tagged template (we only see the literal
        // node, but we can check whether this template is already tagged by
        // checking for `String.raw` in the source text immediately before).
        // A more robust check would use parent tracking, but for now we check
        // whether the source text preceding the template contains `String.raw`.
        let start = usize::try_from(template.span.start).unwrap_or(0);
        let prefix = ctx.source_text().get(start.saturating_sub(11)..start);
        if prefix.is_some_and(|p| p.contains("String.raw")) {
            return;
        }

        let source = ctx.source_text();
        let tmpl_start = usize::try_from(template.span.start).unwrap_or(0);
        let tmpl_end = usize::try_from(template.span.end).unwrap_or(0);
        let tmpl_text = source.get(tmpl_start..tmpl_end).unwrap_or("");

        ctx.report(Diagnostic {
            rule_name: "prefer-string-raw".to_owned(),
            message: "Template literal with escape sequences could use `String.raw`".to_owned(),
            span: Span::new(template.span.start, template.span.end),
            severity: Severity::Warning,
            help: Some("Prefix with `String.raw`".to_owned()),
            fix: Some(Fix {
                message: "Prefix with `String.raw`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(template.span.start, template.span.end),
                    replacement: format!("String.raw{tmpl_text}"),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStringRaw)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_escape_in_template() {
        let diags = lint(r"var x = `foo\nbar`;");
        assert_eq!(diags.len(), 1, "template with escape should be flagged");
    }

    #[test]
    fn test_allows_plain_template() {
        let diags = lint("var x = `hello`;");
        assert!(diags.is_empty(), "plain template should not be flagged");
    }

    #[test]
    fn test_allows_template_with_expression() {
        let diags = lint(r"var x = `hello ${name}\n`;");
        assert!(
            diags.is_empty(),
            "template with expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_already_tagged() {
        let diags = lint(r"var x = String.raw`foo\nbar`;");
        assert!(
            diags.is_empty(),
            "already tagged with String.raw should not be flagged"
        );
    }
}
