//! Rule: `jsx-a11y/aria-proptypes`
//!
//! Enforce ARIA state and property values are valid.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/aria-proptypes";

/// ARIA attributes that accept only `true` or `false`.
const BOOLEAN_ARIA_PROPS: &[&str] = &[
    "aria-atomic",
    "aria-busy",
    "aria-disabled",
    "aria-grabbed",
    "aria-hidden",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-readonly",
    "aria-required",
    "aria-selected",
];

/// ARIA attributes that accept `true`, `false`, or `mixed`.
const TRISTATE_ARIA_PROPS: &[&str] = &["aria-checked", "aria-pressed"];

#[derive(Debug)]
pub struct AriaProptypes;

impl NativeRule for AriaProptypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce ARIA state and property values are valid".to_owned(),
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

                if !name_str.starts_with("aria-") {
                    continue;
                }

                let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value else {
                    continue;
                };

                let val = lit.value.as_str();

                if BOOLEAN_ARIA_PROPS.contains(&name_str) && val != "true" && val != "false" {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("`{name_str}` must be `\"true\"` or `\"false\"`"),
                        span: Span::new(opening.span.start, opening.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }

                if TRISTATE_ARIA_PROPS.contains(&name_str)
                    && val != "true"
                    && val != "false"
                    && val != "mixed"
                {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "`{name_str}` must be `\"true\"`, `\"false\"`, or `\"mixed\"`"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AriaProptypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_invalid_boolean_aria() {
        let diags = lint(r#"const el = <div aria-hidden="yes">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_boolean_aria() {
        let diags = lint(r#"const el = <div aria-hidden="true">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_tristate_mixed() {
        let diags = lint(r#"const el = <div aria-checked="mixed">content</div>;"#);
        assert!(diags.is_empty());
    }
}
