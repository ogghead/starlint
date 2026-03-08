//! Rule: `number-literal-case`
//!
//! Enforce consistent case for numeric literal prefixes and digits.
//! Hex digits should be uppercase (`0xFF`), prefixes lowercase (`0x`),
//! and exponential notation lowercase (`1e3`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags numeric literals with inconsistent casing.
#[derive(Debug)]
pub struct NumberLiteralCase;

/// Normalize a numeric literal to the canonical case form.
///
/// Returns `None` if already canonical (no fix needed) or if the literal
/// is a plain decimal with no special prefix/exponent.
fn normalize(raw: &str) -> Option<String> {
    // Must have a prefix (0x, 0o, 0b) or exponent (e/E) to be relevant.
    if raw.len() < 2 {
        return None;
    }

    let bytes = raw.as_bytes();

    // Check for 0x/0X, 0o/0O, 0b/0B prefix.
    if bytes.first() == Some(&b'0') && bytes.len() > 2 {
        match bytes.get(1) {
            Some(b'x' | b'X') => {
                // Hex: prefix must be lowercase `0x`, digits uppercase.
                let prefix = "0x";
                let digits = &raw[2..];
                let normalized_digits: String = digits
                    .chars()
                    .map(|c| {
                        if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                            c.to_ascii_uppercase()
                        } else {
                            c
                        }
                    })
                    .collect();
                let result = format!("{prefix}{normalized_digits}");
                if result == raw {
                    return None;
                }
                return Some(result);
            }
            Some(b'o' | b'O') => {
                // Octal: prefix must be lowercase `0o`.
                let result = format!("0o{}", &raw[2..]);
                if result == raw {
                    return None;
                }
                return Some(result);
            }
            Some(b'b' | b'B') => {
                // Binary: prefix must be lowercase `0b`.
                let result = format!("0b{}", &raw[2..]);
                if result == raw {
                    return None;
                }
                return Some(result);
            }
            _ => {}
        }
    }

    // Check for exponent: `E` should be `e`.
    if let Some(e_pos) = raw.find('E') {
        // Make sure it's not inside a hex literal (already handled above).
        let mut result = raw.to_owned();
        result.replace_range(e_pos..e_pos.saturating_add(1), "e");
        if result == raw {
            return None;
        }
        return Some(result);
    }

    None
}

impl LintRule for NumberLiteralCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "number-literal-case".to_owned(),
            description: "Enforce consistent case for numeric literals".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NumericLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NumericLiteral(lit) = node else {
            return;
        };

        let start = usize::try_from(lit.span.start).unwrap_or(0);
        let end = usize::try_from(lit.span.end).unwrap_or(0);
        let Some(raw) = ctx.source_text().get(start..end) else {
            return;
        };

        let Some(normalized) = normalize(raw) else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "number-literal-case".to_owned(),
            message: format!("Inconsistent casing in numeric literal `{raw}`"),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace with `{normalized}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{raw}` with `{normalized}`"),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement: normalized,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NumberLiteralCase)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_lowercase_hex_digits() {
        let diags = lint("const x = 0xff;");
        assert_eq!(diags.len(), 1, "should flag 0xff");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("0xFF"),
            "fix should uppercase hex digits"
        );
    }

    #[test]
    fn test_flags_uppercase_hex_prefix() {
        let diags = lint("const x = 0XFF;");
        assert_eq!(diags.len(), 1, "should flag 0XFF");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("0xFF"),
            "fix should lowercase prefix and keep uppercase digits"
        );
    }

    #[test]
    fn test_flags_uppercase_binary_prefix() {
        let diags = lint("const x = 0B0101;");
        assert_eq!(diags.len(), 1, "should flag 0B0101");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("0b0101"),
            "fix should lowercase binary prefix"
        );
    }

    #[test]
    fn test_flags_uppercase_octal_prefix() {
        let diags = lint("const x = 0O777;");
        assert_eq!(diags.len(), 1, "should flag 0O777");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("0o777"),
            "fix should lowercase octal prefix"
        );
    }

    #[test]
    fn test_flags_uppercase_exponent() {
        let diags = lint("const x = 1E3;");
        assert_eq!(diags.len(), 1, "should flag 1E3");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("1e3"),
            "fix should lowercase exponent"
        );
    }

    #[test]
    fn test_allows_correct_hex() {
        let diags = lint("const x = 0xFF;");
        assert!(diags.is_empty(), "0xFF should not be flagged");
    }

    #[test]
    fn test_allows_correct_binary() {
        let diags = lint("const x = 0b0101;");
        assert!(diags.is_empty(), "0b0101 should not be flagged");
    }

    #[test]
    fn test_allows_correct_octal() {
        let diags = lint("const x = 0o777;");
        assert!(diags.is_empty(), "0o777 should not be flagged");
    }

    #[test]
    fn test_allows_plain_decimal() {
        let diags = lint("const x = 42;");
        assert!(diags.is_empty(), "plain decimal should not be flagged");
    }

    #[test]
    fn test_allows_lowercase_exponent() {
        let diags = lint("const x = 1e3;");
        assert!(diags.is_empty(), "1e3 should not be flagged");
    }
}
