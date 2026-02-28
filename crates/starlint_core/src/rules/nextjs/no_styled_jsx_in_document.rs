//! Rule: `nextjs/no-styled-jsx-in-document`
//!
//! Forbid styled-jsx in `_document`. The `<style jsx>` component does not
//! work correctly in `_document` because it is rendered on the server only.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-styled-jsx-in-document";

/// Flags `<style jsx>` elements in `_document` files.
#[derive(Debug)]
pub struct NoStyledJsxInDocument;

/// Get the attribute name as a string.
fn attr_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(ident) => ident.name.as_str(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.as_str(),
    }
}

impl NativeRule for NoStyledJsxInDocument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid styled-jsx in `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Only check in _document files
        let file_stem = ctx
            .file_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_stem != "_document" {
            return;
        }

        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let is_style = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "style",
            _ => false,
        };
        if !is_style {
            return;
        }

        // Check for `jsx` attribute (boolean attribute)
        let has_jsx_attr = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                return attr_name(&attr.name) == "jsx";
            }
            false
        });

        if has_jsx_attr {
            ctx.report_error(
                RULE_NAME,
                "Styled-jsx (`<style jsx>`) should not be used in `_document`",
                Span::new(opening.span.start, opening.span.end),
            );
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoStyledJsxInDocument)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_styled_jsx_in_document() {
        let diags = lint_with_path(
            r"const el = <style jsx>{`.red { color: red; }`}</style>;",
            Path::new("pages/_document.tsx"),
        );
        assert_eq!(diags.len(), 1, "styled-jsx in _document should be flagged");
    }

    #[test]
    fn test_allows_styled_jsx_in_page() {
        let diags = lint_with_path(
            r"const el = <style jsx>{`.red { color: red; }`}</style>;",
            Path::new("pages/index.tsx"),
        );
        assert!(diags.is_empty(), "styled-jsx in page should pass");
    }

    #[test]
    fn test_allows_regular_style_in_document() {
        let diags = lint_with_path(
            r"const el = <style>{`.red { color: red; }`}</style>;",
            Path::new("pages/_document.tsx"),
        );
        assert!(diags.is_empty(), "regular style in _document should pass");
    }
}
