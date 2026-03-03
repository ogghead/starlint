//! Rule: `prefer-dom-node-text-content`
//!
//! Prefer `textContent` over `innerText`. The `innerText` property triggers
//! a reflow and has quirky whitespace behavior. `textContent` is faster and
//! more predictable.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags access to `innerText`, suggesting `textContent` instead.
#[derive(Debug)]
pub struct PreferDomNodeTextContent;

impl NativeRule for PreferDomNodeTextContent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-text-content".to_owned(),
            description: "Prefer `textContent` over `innerText`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StaticMemberExpression(member) = kind else {
            return;
        };

        if member.property.name.as_str() != "innerText" {
            return;
        }

        ctx.report_warning(
            "prefer-dom-node-text-content",
            "Prefer `textContent` over `innerText` — `innerText` triggers a reflow",
            Span::new(member.span.start, member.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDomNodeTextContent)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_inner_text() {
        let diags = lint("var t = el.innerText;");
        assert_eq!(diags.len(), 1, "el.innerText should be flagged");
    }

    #[test]
    fn test_flags_inner_text_assignment() {
        let diags = lint("el.innerText = 'hello';");
        assert_eq!(diags.len(), 1, "el.innerText assignment should be flagged");
    }

    #[test]
    fn test_allows_text_content() {
        let diags = lint("var t = el.textContent;");
        assert!(diags.is_empty(), "el.textContent should not be flagged");
    }

    #[test]
    fn test_allows_inner_html() {
        let diags = lint("var h = el.innerHTML;");
        assert!(diags.is_empty(), "el.innerHTML should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_property() {
        let diags = lint("var v = el.style;");
        assert!(diags.is_empty(), "el.style should not be flagged");
    }
}
