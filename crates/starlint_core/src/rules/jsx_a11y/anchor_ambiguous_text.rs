//! Rule: `jsx-a11y/anchor-ambiguous-text`
//!
//! Forbid ambiguous link text like "click here" or "read more".

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/anchor-ambiguous-text";

/// Ambiguous phrases that should not be used as standalone link text.
const AMBIGUOUS_PHRASES: &[&str] = &[
    "click here",
    "here",
    "read more",
    "learn more",
    "more",
    "link",
];

#[derive(Debug)]
pub struct AnchorAmbiguousText;

/// Get string value of an attribute if it's a string literal.
fn get_attr_string_value<'a>(
    opening: &'a oxc_ast::ast::JSXOpeningElement<'a>,
    attr_name: &str,
) -> Option<&'a str> {
    for item in &opening.attributes {
        if let JSXAttributeItem::Attribute(attr) = item {
            let matches = match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == attr_name,
                JSXAttributeName::NamespacedName(_) => false,
            };
            if matches {
                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    return Some(lit.value.as_str());
                }
            }
        }
    }
    None
}

/// Check if text is an ambiguous link phrase.
fn is_ambiguous(text: &str) -> bool {
    let normalized = text.trim().to_lowercase();
    AMBIGUOUS_PHRASES.iter().any(|phrase| normalized == *phrase)
}

impl NativeRule for AnchorAmbiguousText {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid ambiguous link text like \"click here\" or \"read more\""
                .to_owned(),
            category: Category::Suggestion,
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

        let is_anchor = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "a",
            _ => false,
        };
        if !is_anchor {
            return;
        }

        // Check aria-label for ambiguous text
        if let Some(label) = get_attr_string_value(opening, "aria-label") {
            if is_ambiguous(label) {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Ambiguous link text \"{label}\". Use text that describes the link destination"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AnchorAmbiguousText)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_ambiguous_aria_label() {
        let diags = lint(r#"const el = <a href="/about" aria-label="click here">x</a>;"#);
        assert_eq!(diags.len(), 1, "should flag ambiguous aria-label");
    }

    #[test]
    fn test_allows_descriptive_aria_label() {
        let diags = lint(r#"const el = <a href="/about" aria-label="About our company">x</a>;"#);
        assert!(diags.is_empty(), "should allow descriptive aria-label");
    }

    #[test]
    fn test_allows_anchor_without_aria_label() {
        let diags = lint(r#"const el = <a href="/about">About</a>;"#);
        assert!(diags.is_empty(), "should allow anchor without aria-label");
    }
}
