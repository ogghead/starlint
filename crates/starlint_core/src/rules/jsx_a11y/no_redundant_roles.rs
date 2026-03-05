//! Rule: `jsx-a11y/no-redundant-roles`
//!
//! Forbid redundant roles (e.g., `<button role="button">`).

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-redundant-roles";

/// Mapping of elements to their implicit ARIA roles.
const DEFAULT_ROLE_MAP: &[(&str, &str)] = &[
    ("a", "link"),
    ("article", "article"),
    ("aside", "complementary"),
    ("button", "button"),
    ("footer", "contentinfo"),
    ("form", "form"),
    ("header", "banner"),
    ("img", "img"),
    ("li", "listitem"),
    ("main", "main"),
    ("nav", "navigation"),
    ("ol", "list"),
    ("section", "region"),
    ("table", "table"),
    ("td", "cell"),
    ("textarea", "textbox"),
    ("th", "columnheader"),
    ("tr", "row"),
    ("ul", "list"),
];

#[derive(Debug)]
pub struct NoRedundantRoles;

/// Get the default role for an element.
fn default_role(element: &str) -> Option<&'static str> {
    for &(elem, role) in DEFAULT_ROLE_MAP {
        if elem == element {
            return Some(role);
        }
    }
    None
}

impl NativeRule for NoRedundantRoles {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid redundant roles (e.g., `<button role=\"button\">`)".to_owned(),
            category: Category::Style,
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

        let Some(implicit_role) = default_role(element_name) else {
            return;
        };

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let is_role = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "role",
                    JSXAttributeName::NamespacedName(_) => false,
                };

                if !is_role {
                    continue;
                }

                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    let role_val = lit.value.as_str().trim();
                    if role_val == implicit_role {
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "`<{element_name}>` has an implicit `role` of `{implicit_role}`. Setting `role=\"{role_val}\"` is redundant"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRedundantRoles)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_redundant_button_role() {
        let diags = lint(r#"const el = <button role="button">click</button>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_redundant_nav_role() {
        let diags = lint(r#"const el = <nav role="navigation">menu</nav>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_different_role() {
        let diags = lint(r#"const el = <div role="button">click</div>;"#);
        assert!(diags.is_empty());
    }
}
