//! Rule: `jsx-a11y/role-supports-aria-props`
//!
//! Enforce aria-* props are supported by the element's role.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/role-supports-aria-props";

/// Roles and aria props that they explicitly do NOT support.
/// The `presentation` and `none` roles should have no aria-* props at all.
const ROLES_WITHOUT_ARIA: &[&str] = &["presentation", "none"];

/// Global ARIA props supported by all roles.
const GLOBAL_ARIA_PROPS: &[&str] = &[
    "aria-atomic",
    "aria-busy",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-live",
    "aria-owns",
    "aria-relevant",
    "aria-roledescription",
];

#[derive(Debug)]
pub struct RoleSupportAriaProps;

impl NativeRule for RoleSupportAriaProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce aria-* props are supported by the element's role".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
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

        // presentation/none roles should not have aria-* props (except global ones in some specs)
        if !ROLES_WITHOUT_ARIA.contains(&role) {
            return;
        }

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let name_str = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str(),
                    JSXAttributeName::NamespacedName(_) => continue,
                };

                if name_str.starts_with("aria-") && !GLOBAL_ARIA_PROPS.contains(&name_str) {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("`{name_str}` is not supported by `role=\"{role}\"`"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RoleSupportAriaProps)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_aria_checked_on_presentation() {
        let diags =
            lint(r#"const el = <div role="presentation" aria-checked="true">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_aria_label_on_presentation() {
        let diags =
            lint(r#"const el = <div role="presentation" aria-label="hello">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_aria_on_button_role() {
        let diags = lint(r#"const el = <div role="button" aria-pressed="true">click</div>;"#);
        assert!(diags.is_empty());
    }
}
