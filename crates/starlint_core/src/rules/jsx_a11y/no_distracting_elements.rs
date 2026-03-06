//! Rule: `jsx-a11y/no-distracting-elements`
//!
//! Forbid `<marquee>` and `<blink>` elements.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-distracting-elements";

/// Distracting element names.
const DISTRACTING_ELEMENTS: &[&str] = &["marquee", "blink"];

#[derive(Debug)]
pub struct NoDistractingElements;

impl NativeRule for NoDistractingElements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<marquee>` and `<blink>` elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if DISTRACTING_ELEMENTS.contains(&element_name) {
            let fix = build_replace_fix(ctx.source_text(), opening, element_name);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`<{element_name}>` is distracting and must not be used"),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `<span>`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Build a fix that replaces both opening and closing tag names with `span`.
#[allow(clippy::as_conversions)] // u32↔usize lossless on 32/64-bit
fn build_replace_fix(
    source: &str,
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    element_name: &str,
) -> Option<Fix> {
    let ident_span = match &opening.name {
        JSXElementName::Identifier(ident) => ident.span(),
        _ => return None,
    };

    let mut edits = vec![Edit {
        span: Span::new(ident_span.start, ident_span.end),
        replacement: "span".to_owned(),
    }];

    // Find the closing tag name in the source after the opening element.
    let opening_end = opening.span.end as usize;
    let close_tag = format!("</{element_name}>");
    if let Some(close_offset) = source.get(opening_end..)?.find(&close_tag) {
        let close_name_start = opening_end.saturating_add(close_offset).saturating_add(2); // skip "</"
        let close_name_end = close_name_start.saturating_add(element_name.len());
        edits.push(Edit {
            span: Span::new(
                u32::try_from(close_name_start).ok()?,
                u32::try_from(close_name_end).ok()?,
            ),
            replacement: "span".to_owned(),
        });
    }

    Some(Fix {
        kind: FixKind::SuggestionFix,
        message: format!("Replace `<{element_name}>` with `<span>`"),
        edits,
        is_snippet: false,
    })
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDistractingElements)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_marquee() {
        let diags = lint(r"const el = <marquee>scrolling text</marquee>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_blink() {
        let diags = lint(r"const el = <blink>blinking text</blink>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_normal_elements() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
