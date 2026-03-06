//! Rule: `jsx-a11y/autocomplete-valid`
//!
//! Enforce `autocomplete` attribute has a valid value.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/autocomplete-valid";

/// Valid autocomplete tokens (HTML spec).
const VALID_AUTOCOMPLETE: &[&str] = &[
    "on",
    "off",
    "name",
    "honorific-prefix",
    "given-name",
    "additional-name",
    "family-name",
    "honorific-suffix",
    "nickname",
    "email",
    "username",
    "new-password",
    "current-password",
    "one-time-code",
    "organization-title",
    "organization",
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
    "cc-name",
    "cc-given-name",
    "cc-additional-name",
    "cc-family-name",
    "cc-number",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-csc",
    "cc-type",
    "transaction-currency",
    "transaction-amount",
    "language",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "sex",
    "tel",
    "tel-country-code",
    "tel-national",
    "tel-area-code",
    "tel-local",
    "tel-extension",
    "impp",
    "url",
    "photo",
];

/// Elements that support the `autocomplete` attribute.
const AUTOCOMPLETE_ELEMENTS: &[&str] = &["input", "select", "textarea"];

#[derive(Debug)]
pub struct AutocompleteValid;

impl NativeRule for AutocompleteValid {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `autocomplete` attribute has a valid value".to_owned(),
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

        if !AUTOCOMPLETE_ELEMENTS.contains(&element_name) {
            return;
        }

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let is_autocomplete = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "autoComplete",
                    JSXAttributeName::NamespacedName(_) => false,
                };

                if !is_autocomplete {
                    continue;
                }

                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    let val = lit.value.as_str().trim();
                    // Autocomplete can have section- prefix and shipping/billing qualifiers
                    let tokens: Vec<&str> = val.split_whitespace().collect();
                    if let Some(last) = tokens.last() {
                        if !VALID_AUTOCOMPLETE.contains(last) && !last.starts_with("section-") {
                            let attr_span = Span::new(attr.span.start, attr.span.end);
                            let fix = FixBuilder::new("Remove invalid `autoComplete` attribute")
                                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                                .build();
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: format!("`{val}` is not a valid `autocomplete` value"),
                                span: Span::new(opening.span.start, opening.span.end),
                                severity: Severity::Warning,
                                help: None,
                                fix,
                                labels: vec![],
                            });
                        }
                    }
                }
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AutocompleteValid)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_invalid_autocomplete() {
        let diags = lint(r#"const el = <input autoComplete="foobar" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_autocomplete() {
        let diags = lint(r#"const el = <input autoComplete="email" />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_autocomplete() {
        let diags = lint(r#"const el = <input type="text" />;"#);
        assert!(diags.is_empty());
    }
}
