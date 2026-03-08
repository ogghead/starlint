//! Rule: `no-zero-fractions`
//!
//! Disallow unnecessary zero fractions in numeric literals.
//! `1.0` should be `1`, `1.50` should be `1.5`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags numeric literals with unnecessary zero fractions.
#[derive(Debug)]
pub struct NoZeroFractions;

impl LintRule for NoZeroFractions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-zero-fractions".to_owned(),
            description: "Disallow unnecessary zero fractions in numeric literals".to_owned(),
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

        // Skip scientific notation — `1.0e3` is a different concern.
        if raw.contains('e') || raw.contains('E') {
            return;
        }

        // Must contain a decimal point.
        let Some(dot_pos) = raw.find('.') else {
            return;
        };

        let integer_part = &raw[..dot_pos];
        let decimal_part = &raw[dot_pos.saturating_add(1)..];

        // Compute the trimmed form.
        let trimmed = decimal_part.trim_end_matches('0');

        // No change needed if decimal part has no trailing zeros.
        if trimmed.len() == decimal_part.len() {
            return;
        }

        let replacement = if trimmed.is_empty() {
            // All zeros after dot: `1.0`, `1.00` → `1`
            integer_part.to_owned()
        } else {
            // Trailing zeros: `1.50`, `1.100` → `1.5`, `1.1`
            format!("{integer_part}.{trimmed}")
        };

        ctx.report(Diagnostic {
            rule_name: "no-zero-fractions".to_owned(),
            message: format!("Unnecessary zero fraction in `{raw}`"),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace with `{replacement}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{raw}` with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoZeroFractions)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_dot_zero() {
        let diags = lint("const x = 1.0;");
        assert_eq!(diags.len(), 1, "should flag 1.0");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("1"),
            "fix should remove .0"
        );
    }

    #[test]
    fn test_flags_trailing_zeros() {
        let diags = lint("const x = 1.50;");
        assert_eq!(diags.len(), 1, "should flag 1.50");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("1.5"),
            "fix should trim trailing zeros"
        );
    }

    #[test]
    fn test_flags_zero_dot_zero() {
        let diags = lint("const x = 0.0;");
        assert_eq!(diags.len(), 1, "should flag 0.0");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("0"),
            "fix should be 0"
        );
    }

    #[test]
    fn test_flags_multiple_trailing_zeros() {
        let diags = lint("const x = 1.000;");
        assert_eq!(diags.len(), 1, "should flag 1.000");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("1"),
            "fix should remove all trailing zeros"
        );
    }

    #[test]
    fn test_allows_meaningful_decimals() {
        let diags = lint("const x = 1.5;");
        assert!(diags.is_empty(), "1.5 should not be flagged");
    }

    #[test]
    fn test_allows_integer() {
        let diags = lint("const x = 42;");
        assert!(diags.is_empty(), "integer should not be flagged");
    }

    #[test]
    fn test_allows_scientific_notation() {
        let diags = lint("const x = 1.0e3;");
        assert!(
            diags.is_empty(),
            "scientific notation should not be flagged"
        );
    }

    #[test]
    fn test_allows_small_decimal() {
        let diags = lint("const x = 0.1;");
        assert!(diags.is_empty(), "0.1 should not be flagged");
    }
}
