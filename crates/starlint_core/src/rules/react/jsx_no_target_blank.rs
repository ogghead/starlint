//! Rule: `react/jsx-no-target-blank`
//!
//! Warn when `<a target="_blank">` is used without `rel="noreferrer"`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-target-blank";

/// Flags `<a target="_blank">` elements that are missing `rel="noreferrer"`,
/// which is a security concern (the opened page gains access to `window.opener`).
#[derive(Debug)]
pub struct JsxNoTargetBlank;

/// Get string value from a JSX attribute value if it is a string literal.
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

impl NativeRule for JsxNoTargetBlank {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when `<a target=\"_blank\">` is missing `rel=\"noreferrer\"`"
                .to_owned(),
            category: Category::Correctness,
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

        // Only check `<a>` elements
        let is_anchor = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "a",
            _ => false,
        };
        if !is_anchor {
            return;
        }

        // Check for target="_blank"
        let has_target_blank = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "target" {
                    return get_string_value(attr.value.as_ref()) == Some("_blank");
                }
            }
            false
        });

        if !has_target_blank {
            return;
        }

        // Check for rel containing "noreferrer"
        let has_noreferrer = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                if attr_name(&attr.name) == "rel" {
                    if let Some(val) = get_string_value(attr.value.as_ref()) {
                        return val.split_whitespace().any(|part| part == "noreferrer");
                    }
                }
            }
            false
        });

        if !has_noreferrer {
            ctx.report_warning(
                RULE_NAME,
                "Using `target=\"_blank\"` without `rel=\"noreferrer\"` is a security risk",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoTargetBlank)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_target_blank_without_rel() {
        let diags = lint(r#"const el = <a href="https://example.com" target="_blank">link</a>;"#);
        assert_eq!(diags.len(), 1, "should flag missing rel=noreferrer");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_with_noreferrer() {
        let diags = lint(
            r#"const el = <a href="https://example.com" target="_blank" rel="noreferrer">link</a>;"#,
        );
        assert!(diags.is_empty(), "should not flag when noreferrer present");
    }

    #[test]
    fn test_allows_no_target_blank() {
        let diags = lint(r#"const el = <a href="https://example.com">link</a>;"#);
        assert!(
            diags.is_empty(),
            "should not flag anchor without target=_blank"
        );
    }
}
