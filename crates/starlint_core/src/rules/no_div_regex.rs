//! Rule: `no-div-regex`
//!
//! Disallow regular expressions that look like division. A regex like
//! `/=foo/` can be confused with a division assignment and should be
//! written as `/[=]foo/` or `new RegExp("=foo")`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags regex literals that start with `=`.
#[derive(Debug)]
pub struct NoDivRegex;

impl NativeRule for NoDivRegex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-div-regex".to_owned(),
            description: "Disallow regular expressions that look like division".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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

        if pattern.starts_with('=') {
            // Fix: escape the leading = by wrapping in char class [=]
            let source = ctx.source_text();
            let start = usize::try_from(regex.span.start).unwrap_or(0);
            let end = usize::try_from(regex.span.end).unwrap_or(0);
            let raw = source.get(start..end).unwrap_or("");
            // raw is "/=pattern/flags" — insert [=] after first /
            let fix = raw.get(2..).map(|rest| Fix {
                message: "Escape leading `=` in regex".to_owned(),
                edits: vec![Edit {
                    span: Span::new(regex.span.start, regex.span.end),
                    replacement: format!("/[=]{rest}"),
                }],
            });

            ctx.report(Diagnostic {
                rule_name: "no-div-regex".to_owned(),
                message: "Ambiguous regex: looks like it could be a division operator".to_owned(),
                span: Span::new(regex.span.start, regex.span.end),
                severity: Severity::Warning,
                help: Some("Escape leading `=` with `[=]`".to_owned()),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDivRegex)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_div_like_regex() {
        let diags = lint("var r = /=foo/;");
        assert_eq!(diags.len(), 1, "/=foo/ should be flagged");
    }

    #[test]
    fn test_allows_normal_regex() {
        let diags = lint("var r = /foo/;");
        assert!(diags.is_empty(), "normal regex should not be flagged");
    }

    #[test]
    fn test_allows_char_class_regex() {
        let diags = lint("var r = /[=]foo/;");
        assert!(
            diags.is_empty(),
            "regex with = in char class should not be flagged"
        );
    }
}
