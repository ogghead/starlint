//! Rule: `jsx-a11y/prefer-tag-over-role`
//!
//! Prefer using semantic HTML tags over ARIA roles.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/prefer-tag-over-role";

/// Mapping of ARIA roles to preferred semantic HTML tags.
const ROLE_TO_TAG: &[(&str, &str)] = &[
    ("banner", "<header>"),
    ("button", "<button>"),
    ("cell", "<td>"),
    ("columnheader", "<th>"),
    ("complementary", "<aside>"),
    ("contentinfo", "<footer>"),
    ("form", "<form>"),
    ("heading", "<h1>-<h6>"),
    ("img", "<img>"),
    ("link", "<a>"),
    ("list", "<ul> or <ol>"),
    ("listitem", "<li>"),
    ("main", "<main>"),
    ("navigation", "<nav>"),
    ("row", "<tr>"),
    ("table", "<table>"),
];

#[derive(Debug)]
pub struct PreferTagOverRole;

/// Get the preferred tag for a given role.
fn preferred_tag(role: &str) -> Option<&'static str> {
    for &(r, tag) in ROLE_TO_TAG {
        if r == role {
            return Some(tag);
        }
    }
    None
}

impl NativeRule for PreferTagOverRole {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer using semantic HTML tags over ARIA roles".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
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
                    let role = lit.value.as_str().trim();
                    if let Some(tag) = preferred_tag(role) {
                        ctx.report_warning(
                            RULE_NAME,
                            &format!(
                                "Prefer using the `{tag}` element instead of `role=\"{role}\"`"
                            ),
                            Span::new(opening.span.start, opening.span.end),
                        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferTagOverRole)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_role_button_on_div() {
        let diags = lint(r#"const el = <div role="button">click</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_role_navigation() {
        let diags = lint(r#"const el = <div role="navigation">menu</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_custom_role() {
        let diags = lint(r#"const el = <div role="dialog">content</div>;"#);
        assert!(diags.is_empty());
    }
}
