//! Rule: `nextjs/next-script-for-ga`
//!
//! Suggest using `next/script` for Google Analytics instead of a raw
//! `<script>` element to benefit from Next.js script optimization.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/next-script-for-ga";

/// Known Google Analytics URL patterns.
const GA_PATTERNS: &[&str] = &[
    "www.google-analytics.com/analytics.js",
    "www.googletagmanager.com/gtag/js",
    "googletagmanager.com/gtm.js",
];

/// Flags raw `<script>` elements that load Google Analytics.
#[derive(Debug)]
pub struct NextScriptForGa;

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

impl NativeRule for NextScriptForGa {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Use `next/script` for Google Analytics".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Only check lowercase `<script>` (HTML element, not Next.js `<Script>`)
        let is_script = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "script",
            _ => false,
        };
        if !is_script {
            return;
        }

        // Check if src attribute contains a GA URL
        let has_ga_src = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "src" {
                    if let Some(val) = get_string_value(attr.value.as_ref()) {
                        return GA_PATTERNS.iter().any(|pattern| val.contains(pattern));
                    }
                }
            }
            false
        });

        if has_ga_src {
            ctx.report_warning(
                RULE_NAME,
                "Use the `<Script>` component from `next/script` for Google Analytics instead of a raw `<script>` element",
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NextScriptForGa)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_ga_script() {
        let diags =
            lint(r#"const el = <script src="https://www.google-analytics.com/analytics.js" />;"#);
        assert_eq!(diags.len(), 1, "GA script should be flagged");
    }

    #[test]
    fn test_flags_gtag_script() {
        let diags = lint(
            r#"const el = <script src="https://www.googletagmanager.com/gtag/js?id=G-123" />;"#,
        );
        assert_eq!(diags.len(), 1, "gtag script should be flagged");
    }

    #[test]
    fn test_allows_non_ga_script() {
        let diags = lint(r#"const el = <script src="/my-script.js" />;"#);
        assert!(diags.is_empty(), "non-GA script should not be flagged");
    }
}
