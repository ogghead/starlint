//! Rule: `nextjs/no-unwanted-polyfillio`
//!
//! Forbid polyfill.io scripts. The polyfill.io domain has been compromised
//! and should not be used. Next.js already includes necessary polyfills.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-unwanted-polyfillio";

/// Flags `<script>` elements that load from polyfill.io.
#[derive(Debug)]
pub struct NoUnwantedPolyfillio;

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

impl NativeRule for NoUnwantedPolyfillio {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid polyfill.io scripts".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Error,
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

        let is_script = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "script",
            _ => false,
        };
        if !is_script {
            return;
        }

        let has_polyfill_src = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "src" {
                    if let Some(val) = get_string_value(attr.value.as_ref()) {
                        return val.contains("polyfill.io") || val.contains("polyfill.min.js");
                    }
                }
            }
            false
        });

        if has_polyfill_src {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use polyfill.io -- it has been compromised. Next.js already includes necessary polyfills".to_owned(),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnwantedPolyfillio)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_polyfill_io() {
        let diags =
            lint(r#"const el = <script src="https://cdn.polyfill.io/v3/polyfill.min.js" />;"#);
        assert_eq!(diags.len(), 1, "polyfill.io script should be flagged");
    }

    #[test]
    fn test_allows_other_scripts() {
        let diags = lint(r#"const el = <script src="/my-script.js" />;"#);
        assert!(diags.is_empty(), "other scripts should not be flagged");
    }

    #[test]
    fn test_allows_script_component() {
        let diags = lint(r#"const el = <Script src="/my-script.js" />;"#);
        assert!(diags.is_empty(), "Script component should not be flagged");
    }
}
