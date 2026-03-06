//! Rule: `nextjs/no-before-interactive-script-outside-document`
//!
//! Forbid `strategy="beforeInteractive"` on `<Script>` outside of `_document`.
//! The `beforeInteractive` strategy only works in `pages/_document`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-before-interactive-script-outside-document";

/// Flags `<Script strategy="beforeInteractive">` outside of `_document` files.
#[derive(Debug)]
pub struct NoBeforeInteractiveScriptOutsideDocument;

/// Get the element name string from a JSX element name, handling both
/// `Identifier` (lowercase HTML) and `IdentifierReference` (`PascalCase` components).
fn element_name<'a>(name: &'a JSXElementName<'a>) -> Option<&'a str> {
    match name {
        JSXElementName::Identifier(ident) => Some(ident.name.as_str()),
        JSXElementName::IdentifierReference(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

/// Get string value from a JSX attribute value.
fn get_string_value<'a>(value: Option<&'a JSXAttributeValue<'a>>) -> Option<&'a str> {
    match value {
        Some(JSXAttributeValue::StringLiteral(lit)) => Some(lit.value.as_str()),
        _ => None,
    }
}

/// Get the attribute name as a string.
fn attr_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(ident) => ident.name.as_str(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.as_str(),
    }
}

impl NativeRule for NoBeforeInteractiveScriptOutsideDocument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `strategy=\"beforeInteractive\"` outside `_document`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Only check `<Script>` (PascalCase component)
        if element_name(&opening.name) != Some("Script") {
            return;
        }

        // Check for strategy="beforeInteractive"
        let has_before_interactive = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "strategy" {
                    return get_string_value(attr.value.as_ref()) == Some("beforeInteractive");
                }
            }
            false
        });

        if !has_before_interactive {
            return;
        }

        // Check if the file is _document
        let file_path = ctx.file_path();
        let file_stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        if file_stem != "_document" {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`strategy=\"beforeInteractive\"` on `<Script>` is only allowed in `pages/_document`".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Error,
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> =
                vec![Box::new(NoBeforeInteractiveScriptOutsideDocument)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_before_interactive_outside_document() {
        let diags = lint_with_path(
            r#"const el = <Script strategy="beforeInteractive" src="/script.js" />;"#,
            Path::new("pages/index.tsx"),
        );
        assert_eq!(
            diags.len(),
            1,
            "beforeInteractive outside _document should be flagged"
        );
    }

    #[test]
    fn test_allows_before_interactive_in_document() {
        let diags = lint_with_path(
            r#"const el = <Script strategy="beforeInteractive" src="/script.js" />;"#,
            Path::new("pages/_document.tsx"),
        );
        assert!(
            diags.is_empty(),
            "beforeInteractive in _document should pass"
        );
    }

    #[test]
    fn test_allows_other_strategies() {
        let diags = lint_with_path(
            r#"const el = <Script strategy="afterInteractive" src="/script.js" />;"#,
            Path::new("pages/index.tsx"),
        );
        assert!(diags.is_empty(), "other strategies should not be flagged");
    }
}
