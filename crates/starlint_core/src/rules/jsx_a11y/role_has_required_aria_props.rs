//! Rule: `jsx-a11y/role-has-required-aria-props`
//!
//! Enforce elements with ARIA roles have required aria-* props.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/role-has-required-aria-props";

/// Roles and their required ARIA properties.
const ROLE_REQUIRED_PROPS: &[(&str, &[&str])] = &[
    ("checkbox", &["aria-checked"]),
    ("combobox", &["aria-expanded"]),
    ("heading", &["aria-level"]),
    ("meter", &["aria-valuenow"]),
    ("option", &["aria-selected"]),
    ("radio", &["aria-checked"]),
    ("scrollbar", &["aria-controls", "aria-valuenow"]),
    ("separator", &["aria-valuenow"]),
    ("slider", &["aria-valuenow"]),
    ("spinbutton", &["aria-valuenow"]),
    ("switch", &["aria-checked"]),
];

#[derive(Debug)]
pub struct RoleHasRequiredAriaProps;

/// Check if an attribute exists on a JSX element.
fn has_attribute(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> bool {
    opening.attributes.iter().any(|item| {
        if let JSXAttributeItem::Attribute(attr) = item {
            match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == name,
                JSXAttributeName::NamespacedName(_) => false,
            }
        } else {
            false
        }
    })
}

/// Get the required aria props for a given role.
fn required_props(role: &str) -> Option<&'static [&'static str]> {
    for &(r, props) in ROLE_REQUIRED_PROPS {
        if r == role {
            return Some(props);
        }
    }
    None
}

impl NativeRule for RoleHasRequiredAriaProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce elements with ARIA roles have required aria-* props".to_owned(),
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

        // Find the role attribute value
        let mut role_value: Option<&str> = None;
        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let is_role = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "role",
                    JSXAttributeName::NamespacedName(_) => false,
                };

                if is_role {
                    if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                        role_value = Some(lit.value.as_str());
                    }
                    break;
                }
            }
        }

        let Some(role_raw) = role_value else {
            return;
        };

        let role = role_raw.trim();
        let Some(props) = required_props(role) else {
            return;
        };

        for prop in props {
            if !has_attribute(opening, prop) {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Elements with `role=\"{role}\"` must have the `{prop}` attribute"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RoleHasRequiredAriaProps)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_checkbox_without_aria_checked() {
        let diags = lint(r#"const el = <div role="checkbox">check</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_checkbox_with_aria_checked() {
        let diags = lint(r#"const el = <div role="checkbox" aria-checked="true">check</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_role() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
