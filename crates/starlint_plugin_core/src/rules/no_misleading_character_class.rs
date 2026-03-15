//! Rule: `no-misleading-character-class` (eslint)
//!
//! Disallow characters which are made with multiple code points in character
//! class syntax. Characters like `👶🏻` (emoji with skin tone modifier) look
//! like a single character but are composed of multiple code points, and
//! character classes in regex match individual code points.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags character classes containing multi-code-point characters.
#[derive(Debug)]
pub struct NoMisleadingCharacterClass;

impl LintRule for NoMisleadingCharacterClass {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-misleading-character-class".to_owned(),
            description: "Disallow multi-code-point characters in character classes".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::RegExpLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::RegExpLiteral(regex) = node else {
            return;
        };

        let pattern = regex.pattern.as_str();

        if has_misleading_char_class(pattern) {
            ctx.report(Diagnostic {
                rule_name: "no-misleading-character-class".to_owned(),
                message: "Character class contains a character composed of multiple code points"
                    .to_owned(),
                span: Span::new(regex.span.start, regex.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a regex pattern has character classes with combining marks or surrogates.
fn has_misleading_char_class(pattern: &str) -> bool {
    let mut in_char_class = false;
    let mut prev_was_high_surrogate = false;
    let mut chars = pattern.chars();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Skip escaped character
                let _ = chars.next();
                prev_was_high_surrogate = false;
            }
            '[' if !in_char_class => {
                in_char_class = true;
                prev_was_high_surrogate = false;
            }
            ']' if in_char_class => {
                in_char_class = false;
                prev_was_high_surrogate = false;
            }
            _ if in_char_class => {
                // Check for combining marks after a base character
                if is_combining_mark(ch) && prev_was_high_surrogate {
                    return true;
                }
                // Check for variation selectors (emoji modifiers)
                if is_variation_selector(ch) {
                    return true;
                }
                // Check for zero-width joiners in char classes
                if ch == '\u{200D}' {
                    return true;
                }
                prev_was_high_surrogate = !ch.is_ascii();
            }
            _ => {
                prev_was_high_surrogate = false;
            }
        }
    }

    false
}

/// Check if a character is a Unicode combining mark.
#[allow(clippy::as_conversions)] // char → u32 is lossless; u32::from not const-stable
const fn is_combining_mark(ch: char) -> bool {
    let cp = ch as u32;
    // Combining Diacritical Marks: U+0300 – U+036F
    // Combining Diacritical Marks Extended: U+1AB0 – U+1AFF
    // Combining Diacritical Marks Supplement: U+1DC0 – U+1DFF
    // Combining Half Marks: U+FE20 – U+FE2F
    (cp >= 0x0300 && cp <= 0x036F)
        || (cp >= 0x1AB0 && cp <= 0x1AFF)
        || (cp >= 0x1DC0 && cp <= 0x1DFF)
        || (cp >= 0xFE20 && cp <= 0xFE2F)
        // Skin tone modifiers
        || (cp >= 0x1F3FB && cp <= 0x1F3FF)
}

/// Check if a character is a variation selector.
#[allow(clippy::as_conversions)] // char → u32 is lossless; u32::from not const-stable
const fn is_variation_selector(ch: char) -> bool {
    let cp = ch as u32;
    // Variation Selectors: U+FE00 – U+FE0F
    // Variation Selectors Supplement: U+E0100 – U+E01EF
    (cp >= 0xFE00 && cp <= 0xFE0F) || (cp >= 0xE0100 && cp <= 0xE01EF)
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoMisleadingCharacterClass);

    #[test]
    fn test_allows_simple_char_class() {
        let diags = lint("var re = /[abc]/;");
        assert!(
            diags.is_empty(),
            "simple character class should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_char_class() {
        let diags = lint("var re = /abc/;");
        assert!(
            diags.is_empty(),
            "regex without character class should not be flagged"
        );
    }

    #[test]
    fn test_allows_escaped_chars() {
        let diags = lint("var re = /[\\d\\w]/;");
        assert!(
            diags.is_empty(),
            "escaped chars in class should not be flagged"
        );
    }
}
