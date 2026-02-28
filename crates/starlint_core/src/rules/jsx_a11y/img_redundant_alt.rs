//! Rule: `jsx-a11y/img-redundant-alt`
//!
//! Forbid words like "image", "picture", or "photo" in `<img>` alt text.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/img-redundant-alt";

/// Redundant words in alt text for images.
const REDUNDANT_WORDS: &[&str] = &["image", "picture", "photo", "img", "photograph"];

#[derive(Debug)]
pub struct ImgRedundantAlt;

impl NativeRule for ImgRedundantAlt {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid redundant words like \"image\" or \"photo\" in `<img>` alt text"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let is_img = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "img",
            _ => false,
        };

        if !is_img {
            return;
        }

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let is_alt = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "alt",
                    JSXAttributeName::NamespacedName(_) => false,
                };

                if !is_alt {
                    continue;
                }

                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    let alt_lower = lit.value.as_str().to_lowercase();
                    for word in REDUNDANT_WORDS {
                        if alt_lower.contains(word) {
                            ctx.report_warning(
                                RULE_NAME,
                                &format!(
                                    "Redundant alt text. Screen readers already announce `<img>` elements as images. Avoid using \"{word}\""
                                ),
                                Span::new(opening.span.start, opening.span.end),
                            );
                            break;
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ImgRedundantAlt)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_redundant_alt() {
        let diags = lint(r#"const el = <img alt="image of a cat" src="cat.png" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_descriptive_alt() {
        let diags = lint(r#"const el = <img alt="A fluffy orange cat" src="cat.png" />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_photo_in_alt() {
        let diags = lint(r#"const el = <img alt="Photo of sunset" src="sunset.png" />;"#);
        assert_eq!(diags.len(), 1);
    }
}
