//! Rule: `jsx-a11y/no-distracting-elements`
//!
//! Forbid `<marquee>` and `<blink>` elements.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
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

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if DISTRACTING_ELEMENTS.contains(&element_name) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`<{element_name}>` is distracting and must not be used"),
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
