//! Rule: `jsx-a11y/aria-props`
//!
//! Enforce valid `aria-*` attribute names.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-props";

/// All valid WAI-ARIA 1.1 attribute names.
const VALID_ARIA_PROPS: &[&str] = &[
    "aria-activedescendant",
    "aria-atomic",
    "aria-autocomplete",
    "aria-busy",
    "aria-checked",
    "aria-colcount",
    "aria-colindex",
    "aria-colspan",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-expanded",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-level",
    "aria-live",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-orientation",
    "aria-owns",
    "aria-placeholder",
    "aria-posinset",
    "aria-pressed",
    "aria-readonly",
    "aria-relevant",
    "aria-required",
    "aria-roledescription",
    "aria-rowcount",
    "aria-rowindex",
    "aria-rowspan",
    "aria-selected",
    "aria-setsize",
    "aria-sort",
    "aria-valuemax",
    "aria-valuemin",
    "aria-valuenow",
    "aria-valuetext",
];

#[derive(Debug)]
pub struct AriaProps;

impl NativeRule for AriaProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce valid `aria-*` attribute names".to_owned(),
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

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let name_str = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str(),
                    JSXAttributeName::NamespacedName(_) => continue,
                };

                if name_str.starts_with("aria-") && !VALID_ARIA_PROPS.contains(&name_str) {
                    let attr_span = Span::new(attr.span.start, attr.span.end);
                    let fix = FixBuilder::new(
                        format!("Remove invalid `{name_str}` attribute"),
                        FixKind::SafeFix,
                    )
                    .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                    .build();
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("`{name_str}` is not a valid WAI-ARIA attribute"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AriaProps)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_invalid_aria_prop() {
        let diags = lint(r#"const el = <div aria-foobar="true">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_aria_prop() {
        let diags = lint(r#"const el = <div aria-label="hello">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_ignores_non_aria_attributes() {
        let diags = lint(r#"const el = <div data-custom="true">content</div>;"#);
        assert!(diags.is_empty());
    }
}
