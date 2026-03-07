//! Rule: `approx-constant`
//!
//! Flag floating-point literals that approximate well-known `Math` constants.
//! Using `Math.PI` instead of `3.14` is more precise, self-documenting, and
//! less error-prone.
//!
//! Detection uses a string-prefix check on the raw source text, which is more
//! reliable than floating-point comparison for this purpose.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags numeric literals that approximate well-known `Math` constants.
#[derive(Debug)]
pub struct ApproxConstant;

/// A known math constant with its string prefix for matching and the
/// recommended `Math.*` property name.
struct KnownConstant {
    /// The string prefix to match against raw source text (e.g. "3.14").
    prefix: &'static str,
    /// The `Math.*` property to suggest (e.g. "Math.PI").
    name: &'static str,
}

/// Known math constants and their distinguishing prefixes.
///
/// Prefixes are chosen to be long enough to avoid false positives on
/// unrelated values but short enough to catch common approximations.
const KNOWN_CONSTANTS: &[KnownConstant] = &[
    KnownConstant {
        prefix: "3.14",
        name: "Math.PI",
    },
    KnownConstant {
        prefix: "2.718",
        name: "Math.E",
    },
    KnownConstant {
        prefix: "0.693",
        name: "Math.LN2",
    },
    KnownConstant {
        prefix: "2.302",
        name: "Math.LN10",
    },
    KnownConstant {
        prefix: "1.442",
        name: "Math.LOG2E",
    },
    KnownConstant {
        prefix: "0.434",
        name: "Math.LOG10E",
    },
    KnownConstant {
        prefix: "1.414",
        name: "Math.SQRT2",
    },
];

impl LintRule for ApproxConstant {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "approx-constant".to_owned(),
            description: "Flag approximate math constants — use Math.* properties instead"
                .to_owned(),
            category: Category::Suggestion,
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

        let raw_str = lit.raw.as_str();

        // Only check decimal float literals (must contain a decimal point).
        if !raw_str.contains('.') {
            return;
        }

        for constant in KNOWN_CONSTANTS {
            if raw_str.starts_with(constant.prefix) {
                ctx.report(Diagnostic {
                    rule_name: "approx-constant".to_owned(),
                    message: format!(
                        "Approximate value of `{}` found — consider using `{}`",
                        raw_str, constant.name,
                    ),
                    span: Span::new(lit.span.start, lit.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `{}`", constant.name)),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{}`", constant.name),
                        edits: vec![Edit {
                            span: Span::new(lit.span.start, lit.span.end),
                            replacement: constant.name.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
                // Only report the first matching constant.
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ApproxConstant)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_pi_approximation() {
        let diags = lint("const x = 3.14;");
        assert_eq!(diags.len(), 1, "3.14 should be flagged as approx Math.PI");
    }

    #[test]
    fn test_flags_pi_more_precise() {
        let diags = lint("const x = 3.14159;");
        assert_eq!(
            diags.len(),
            1,
            "3.14159 should be flagged as approx Math.PI"
        );
    }

    #[test]
    fn test_flags_e_approximation() {
        let diags = lint("const x = 2.718;");
        assert_eq!(diags.len(), 1, "2.718 should be flagged as approx Math.E");
    }

    #[test]
    fn test_flags_sqrt2_approximation() {
        let diags = lint("const x = 1.414;");
        assert_eq!(
            diags.len(),
            1,
            "1.414 should be flagged as approx Math.SQRT2"
        );
    }

    #[test]
    fn test_flags_ln2_approximation() {
        let diags = lint("const x = 0.693;");
        assert_eq!(diags.len(), 1, "0.693 should be flagged as approx Math.LN2");
    }

    #[test]
    fn test_flags_ln10_approximation() {
        let diags = lint("const x = 2.3025;");
        assert_eq!(
            diags.len(),
            1,
            "2.3025 should be flagged as approx Math.LN10"
        );
    }

    #[test]
    fn test_flags_log2e_approximation() {
        let diags = lint("const x = 1.4426;");
        assert_eq!(
            diags.len(),
            1,
            "1.4426 should be flagged as approx Math.LOG2E"
        );
    }

    #[test]
    fn test_flags_log10e_approximation() {
        let diags = lint("const x = 0.4342;");
        assert_eq!(
            diags.len(),
            1,
            "0.4342 should be flagged as approx Math.LOG10E"
        );
    }

    #[test]
    fn test_allows_integer() {
        let diags = lint("const x = 3;");
        assert!(diags.is_empty(), "integer 3 should not be flagged");
    }

    #[test]
    fn test_allows_non_matching_float() {
        let diags = lint("const x = 3.15;");
        assert!(
            diags.is_empty(),
            "3.15 should not be flagged (not close enough to PI)"
        );
    }

    #[test]
    fn test_allows_unrelated_float() {
        let diags = lint("const x = 1.5;");
        assert!(diags.is_empty(), "1.5 should not match any known constant");
    }

    #[test]
    fn test_allows_zero() {
        let diags = lint("const x = 0.0;");
        assert!(diags.is_empty(), "0.0 should not be flagged");
    }
}
