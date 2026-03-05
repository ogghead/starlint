//! Rule: `nextjs/no-img-element`
//!
//! Forbid `<img>` HTML element, use `next/image` instead for optimized
//! image loading with automatic lazy loading, resizing, and format selection.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-img-element";

/// Flags `<img>` elements.
#[derive(Debug)]
pub struct NoImgElement;

impl NativeRule for NoImgElement {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<img>` HTML element, use `next/image` instead".to_owned(),
            category: Category::Performance,
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

        let is_img = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "img",
            _ => false,
        };

        if is_img {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not use `<img>`, use `next/image` `<Image>` instead for optimized images"
                        .to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImgElement)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_img_element() {
        let diags = lint(r#"const el = <img src="/photo.jpg" alt="photo" />;"#);
        assert_eq!(diags.len(), 1, "<img> should be flagged");
    }

    #[test]
    fn test_allows_image_component() {
        let diags = lint(r#"const el = <Image src="/photo.jpg" alt="photo" />;"#);
        assert!(diags.is_empty(), "<Image> should not be flagged");
    }

    #[test]
    fn test_allows_other_elements() {
        let diags = lint(r"const el = <div>hello</div>;");
        assert!(diags.is_empty(), "other elements should not be flagged");
    }
}
