//! Rule: `empty-brace-spaces`
//!
//! Disallow spaces inside empty object braces. `{ }` should be `{}`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags object expressions with spaces inside empty braces like `{ }`.
#[derive(Debug)]
pub struct EmptyBraceSpaces;

impl NativeRule for EmptyBraceSpaces {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "empty-brace-spaces".to_owned(),
            description: "Disallow spaces inside empty object braces".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ObjectExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ObjectExpression(obj) = kind else {
            return;
        };

        // Only flag empty objects (no properties, no spread).
        if !obj.properties.is_empty() {
            return;
        }

        let start = usize::try_from(obj.span.start).unwrap_or(0);
        let end = usize::try_from(obj.span.end).unwrap_or(0);
        let Some(raw) = ctx.source_text().get(start..end) else {
            return;
        };

        // If the source text is already `{}`, no issue.
        if raw == "{}" {
            return;
        }

        // Check for spaces/whitespace between the braces.
        let inner = &raw[1..raw.len().saturating_sub(1)];
        if !inner.trim().is_empty() {
            // Something other than whitespace between braces — not our concern.
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "empty-brace-spaces".to_owned(),
            message: "Unexpected spaces inside empty braces".to_owned(),
            span: Span::new(obj.span.start, obj.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `{}`".to_owned()),
            fix: Some(Fix {
                message: "Remove spaces inside braces".to_owned(),
                edits: vec![Edit {
                    span: Span::new(obj.span.start, obj.span.end),
                    replacement: "{}".to_owned(),
                }],
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(EmptyBraceSpaces)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_space_in_empty_braces() {
        let diags = lint("const x = { };");
        assert_eq!(diags.len(), 1, "should flag empty braces with space");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("{}"),
            "fix should remove spaces"
        );
    }

    #[test]
    fn test_flags_multiple_spaces() {
        let diags = lint("const x = {   };");
        assert_eq!(
            diags.len(),
            1,
            "should flag empty braces with multiple spaces"
        );
    }

    #[test]
    fn test_allows_empty_no_spaces() {
        let diags = lint("const x = {};");
        assert!(
            diags.is_empty(),
            "empty braces without spaces should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_empty_object() {
        let diags = lint("const x = { a: 1 };");
        assert!(diags.is_empty(), "non-empty object should not be flagged");
    }
}
