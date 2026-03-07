//! Rule: `no-loss-of-precision`
//!
//! Disallow number literals that lose precision when converted to a JavaScript
//! `Number`. JavaScript uses IEEE 754 double-precision floating-point, which
//! can exactly represent integers up to 2^53. Larger integers or decimals with
//! too many significant digits silently lose precision.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags numeric literals that lose precision in IEEE 754 double-precision.
#[derive(Debug)]
pub struct NoLossOfPrecision;

impl LintRule for NoLossOfPrecision {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-loss-of-precision".to_owned(),
            description: "Disallow literal numbers that lose precision".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NumericLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NumericLiteral(lit) = node else {
            return;
        };

        let raw = lit.raw.as_str();

        // Skip non-decimal literals (0x, 0o, 0b) — they are typically
        // exact and handled by other rules.
        if raw.starts_with("0x")
            || raw.starts_with("0X")
            || raw.starts_with("0o")
            || raw.starts_with("0O")
            || raw.starts_with("0b")
            || raw.starts_with("0B")
        {
            return;
        }

        // Parse the raw text as f64, then format it back.
        // If the round-trip doesn't match the original value, precision was lost.
        if loses_precision(raw, lit.value) {
            ctx.report(Diagnostic {
                rule_name: "no-loss-of-precision".to_owned(),
                message: format!("This number literal will lose precision at runtime: `{raw}`"),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a numeric literal loses precision when stored as f64.
///
/// Compare the f64 value round-tripped through string conversion against
/// the original raw text. If they differ, precision was lost.
fn loses_precision(raw: &str, value: f64) -> bool {
    // Strip underscores (numeric separators)
    let clean: String = raw.chars().filter(|c| *c != '_').collect();

    // Parse the clean string as f64
    let Ok(parsed) = clean.parse::<f64>() else {
        return false;
    };

    // If the value doesn't match what oxc parsed, something is wrong — skip
    if (parsed - value).abs() > f64::EPSILON {
        return false;
    }

    // For integers without decimal point or exponent, check if it exceeds
    // safe integer range
    if !clean.contains('.') && !clean.contains('e') && !clean.contains('E') {
        // Parse as i128 to check the exact integer value
        if let Ok(int_val) = clean.parse::<i128>() {
            // Check if it's outside the safe integer range
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            let roundtrip = value as i128;
            return int_val != roundtrip;
        }
    }

    // For floats, check if converting to string and back gives the same value
    // Use the repr format that preserves full precision
    let repr = format!("{value}");
    let Ok(roundtrip) = repr.parse::<f64>() else {
        return false;
    };

    // If the round-trip value differs from the original parsed value, skip
    // (this shouldn't happen but is a safety check)
    if (roundtrip - value).abs() > f64::EPSILON {
        return false;
    }

    // Now check: does the original source text parse to a different number
    // than what we'd get from the canonical representation?
    // A simpler check: for numbers with many digits, see if the string
    // representation of the f64 value has fewer significant digits.
    let orig_sig_digits = count_significant_digits(&clean);
    let repr_sig_digits = count_significant_digits(&repr);

    // If the original has more significant digits than f64 can represent
    // (about 15-17 decimal digits), it likely loses precision
    orig_sig_digits > 17 && repr_sig_digits < orig_sig_digits
}

/// Count the significant digits in a numeric string.
fn count_significant_digits(s: &str) -> usize {
    let mut count: usize = 0;
    let mut started = false;
    let mut in_exponent = false;

    for ch in s.chars() {
        if ch == 'e' || ch == 'E' {
            in_exponent = true;
            continue;
        }
        if in_exponent {
            continue;
        }
        if ch == '-' || ch == '+' {
            continue;
        }
        if ch.is_ascii_digit() {
            if ch != '0' {
                started = true;
            }
            if started {
                count = count.checked_add(1).unwrap_or(count);
            }
        }
        // Skip decimal point
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLossOfPrecision)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_safe_integer() {
        let diags = lint("var x = 42;");
        assert!(diags.is_empty(), "42 should not be flagged");
    }

    #[test]
    fn test_allows_small_float() {
        let diags = lint("var x = 3.14;");
        assert!(diags.is_empty(), "3.14 should not be flagged");
    }

    #[test]
    fn test_allows_max_safe_integer() {
        let diags = lint("var x = 9007199254740991;");
        assert!(
            diags.is_empty(),
            "Number.MAX_SAFE_INTEGER should not be flagged"
        );
    }

    #[test]
    fn test_flags_large_integer() {
        let diags = lint("var x = 123456789012345678901234567890;");
        assert_eq!(diags.len(), 1, "extremely large integer should be flagged");
    }

    #[test]
    fn test_allows_zero() {
        let diags = lint("var x = 0;");
        assert!(diags.is_empty(), "0 should not be flagged");
    }

    #[test]
    fn test_allows_hex() {
        let diags = lint("var x = 0xFF;");
        assert!(diags.is_empty(), "hex literals should not be flagged");
    }
}
