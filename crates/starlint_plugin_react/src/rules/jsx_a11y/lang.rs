//! Rule: `jsx-a11y/lang`
//!
//! Enforce `lang` attribute has a valid value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

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

impl LintRule for Lang {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `lang` attribute has a valid value".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // opening.name is a String
        if opening.name.as_str() != "html" {
            return;
        }

        for attr_id in &*opening.attributes {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                if attr.name.as_str() != "lang" {
                    continue;
                }

                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        let val = lit.value.as_str().trim();
                        if val.is_empty() {
                            // Replace empty lang value with "en"
                            let fix = FixBuilder::new("Set `lang` to `\"en\"`", FixKind::SafeFix)
                                .replace(Span::new(lit.span.start, lit.span.end), "\"en\"")
                                .build();
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: "The `lang` attribute must not be empty".to_owned(),
                                span: Span::new(opening.span.start, opening.span.end),
                                severity: Severity::Warning,
                                help: Some(
                                    "Set `lang` to a valid BCP 47 tag like `\"en\"`".to_owned(),
                                ),
                                fix,
                                labels: vec![],
                            });
                        } else {
                            let primary = primary_subtag(val).to_lowercase();
                            if !VALID_LANG_CODES.contains(&primary.as_str()) {
                                ctx.report(Diagnostic {
                                    rule_name: RULE_NAME.to_owned(),
                                    message: format!("`{val}` is not a valid BCP 47 language tag"),
                                    span: Span::new(opening.span.start, opening.span.end),
                                    severity: Severity::Warning,
                                    help: Some(
                                        "Use a valid BCP 47 language tag like `\"en\"`".to_owned(),
                                    ),
                                    fix: None,
                                    labels: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(Lang)];
        lint_source(source, "test.js", &rules)
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
