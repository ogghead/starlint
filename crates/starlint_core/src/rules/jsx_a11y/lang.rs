//! Rule: `jsx-a11y/lang`
//!
//! Enforce `lang` attribute has a valid value.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/lang";

/// Common valid BCP 47 language tags (primary language subtags).
const VALID_LANG_CODES: &[&str] = &[
    "aa", "ab", "af", "ak", "am", "an", "ar", "as", "av", "ay", "az", "ba", "be", "bg", "bh", "bi",
    "bm", "bn", "bo", "br", "bs", "ca", "ce", "ch", "co", "cr", "cs", "cu", "cv", "cy", "da", "de",
    "dv", "dz", "ee", "el", "en", "eo", "es", "et", "eu", "fa", "ff", "fi", "fj", "fo", "fr", "fy",
    "ga", "gd", "gl", "gn", "gu", "gv", "ha", "he", "hi", "ho", "hr", "ht", "hu", "hy", "hz", "ia",
    "id", "ie", "ig", "ii", "ik", "io", "is", "it", "iu", "ja", "jv", "ka", "kg", "ki", "kj", "kk",
    "kl", "km", "kn", "ko", "kr", "ks", "ku", "kv", "kw", "ky", "la", "lb", "lg", "li", "ln", "lo",
    "lt", "lu", "lv", "mg", "mh", "mi", "mk", "ml", "mn", "mr", "ms", "mt", "my", "na", "nb", "nd",
    "ne", "ng", "nl", "nn", "no", "nr", "nv", "ny", "oc", "oj", "om", "or", "os", "pa", "pi", "pl",
    "ps", "pt", "qu", "rm", "rn", "ro", "ru", "rw", "sa", "sc", "sd", "se", "sg", "si", "sk", "sl",
    "sm", "sn", "so", "sq", "sr", "ss", "st", "su", "sv", "sw", "ta", "te", "tg", "th", "ti", "tk",
    "tl", "tn", "to", "tr", "ts", "tt", "tw", "ty", "ug", "uk", "ur", "uz", "ve", "vi", "vo", "wa",
    "wo", "xh", "yi", "yo", "za", "zh", "zu",
];

#[derive(Debug)]
pub struct Lang;

/// Extract the primary language subtag from a BCP 47 tag.
fn primary_subtag(lang: &str) -> &str {
    lang.split('-').next().unwrap_or(lang)
}

impl NativeRule for Lang {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `lang` attribute has a valid value".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let is_html = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "html",
            _ => false,
        };

        if !is_html {
            return;
        }

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let is_lang = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "lang",
                    JSXAttributeName::NamespacedName(_) => false,
                };

                if !is_lang {
                    continue;
                }

                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    let val = lit.value.as_str().trim();
                    if val.is_empty() {
                        ctx.report_warning(
                            RULE_NAME,
                            "The `lang` attribute must not be empty",
                            Span::new(opening.span.start, opening.span.end),
                        );
                    } else {
                        let primary = primary_subtag(val).to_lowercase();
                        if !VALID_LANG_CODES.contains(&primary.as_str()) {
                            ctx.report_warning(
                                RULE_NAME,
                                &format!("`{val}` is not a valid BCP 47 language tag"),
                                Span::new(opening.span.start, opening.span.end),
                            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Lang)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_invalid_lang() {
        let diags = lint(r#"const el = <html lang="xyz"><body>content</body></html>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_lang() {
        let diags = lint(r#"const el = <html lang="en"><body>content</body></html>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_lang_with_region() {
        let diags = lint(r#"const el = <html lang="en-US"><body>content</body></html>;"#);
        assert!(diags.is_empty());
    }
}
