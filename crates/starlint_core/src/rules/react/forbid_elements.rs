//! Rule: `react/forbid-elements`
//!
//! Warn when forbidden elements are used. Flags `<marquee>` and `<blink>`.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of forbidden HTML elements. By default, `<marquee>` and
/// `<blink>` are flagged as deprecated and non-standard elements.
#[derive(Debug)]
pub struct ForbidElements;

/// Deprecated or non-standard HTML element names that should be avoided.
const FORBIDDEN_ELEMENTS: &[&str] = &["marquee", "blink"];

impl NativeRule for ForbidElements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/forbid-elements".to_owned(),
            description: "Warn when forbidden elements are used".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let tag_name = match &opening.name {
            JSXElementName::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if FORBIDDEN_ELEMENTS.contains(&tag_name) {
            ctx.report(Diagnostic {
                rule_name: "react/forbid-elements".to_owned(),
                message: format!(
                    "`<{tag_name}>` is forbidden — this element is deprecated or non-standard"
                ),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ForbidElements)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_marquee() {
        let source = "const x = <marquee>Scrolling</marquee>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "marquee should be flagged");
    }

    #[test]
    fn test_flags_blink() {
        let source = "const x = <blink>Blinking</blink>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "blink should be flagged");
    }

    #[test]
    fn test_allows_normal_elements() {
        let source = "const x = <div>Hello</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "normal elements should not be flagged");
    }
}
