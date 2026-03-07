//! Rule: `valid-typeof`
//!
//! Enforce comparing `typeof` expressions against valid type strings.
//! The `typeof` operator returns one of: "undefined", "object", "boolean",
//! "number", "string", "function", "symbol", "bigint". Any other comparison
//! is almost certainly a typo.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Valid return values from the `typeof` operator.
const VALID_TYPEOF_VALUES: &[&str] = &[
    "undefined",
    "object",
    "boolean",
    "number",
    "string",
    "function",
    "symbol",
    "bigint",
];

/// Flags `typeof` comparisons against invalid type strings.
#[derive(Debug)]
pub struct ValidTypeof;

impl LintRule for ValidTypeof {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "valid-typeof".to_owned(),
            description: "Enforce comparing `typeof` expressions against valid strings".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only check equality/inequality comparisons
        if !expr.operator.is_equality() {
            return;
        }

        // Check both orderings: typeof x === "..." and "..." === typeof x
        if is_typeof(ctx, expr.left) {
            check_typeof_value(ctx, expr.right, expr.span);
        } else if is_typeof(ctx, expr.right) {
            check_typeof_value(ctx, expr.left, expr.span);
        }
    }
}

/// Check whether a node is a `typeof` unary expression.
fn is_typeof(ctx: &LintContext<'_>, id: NodeId) -> bool {
    matches!(
        ctx.node(id),
        Some(AstNode::UnaryExpression(u)) if u.operator == UnaryOperator::Typeof
    )
}

/// If the other side of the comparison is a string literal, check it's a valid typeof value.
fn check_typeof_value(
    ctx: &mut LintContext<'_>,
    id: NodeId,
    full_span: starlint_ast::types::Span,
) {
    let Some(AstNode::StringLiteral(lit)) = ctx.node(id) else {
        return;
    };

    let value = lit.value.as_str();
    if VALID_TYPEOF_VALUES.contains(&value) {
        return;
    }

    let suggestion = closest_typeof_value(value);
    let fix = suggestion.map(|suggested| {
        Fix {
            kind: FixKind::SafeFix,
            message: format!("Replace with `\"{suggested}\"`"),
            edits: vec![Edit {
                span: Span::new(lit.span.start, lit.span.end),
                replacement: format!("\"{suggested}\""),
            }],
            is_snippet: false,
        }
    });
    let help = suggestion.map(|s| format!("Did you mean `\"{s}\"`?"));
    ctx.report(Diagnostic {
        rule_name: "valid-typeof".to_owned(),
        message: format!("Invalid typeof comparison value `\"{value}\"`"),
        span: Span::new(full_span.start, full_span.end),
        severity: Severity::Error,
        help,
        fix,
        labels: vec![],
    });
}

/// Find the closest valid typeof value using simple edit distance.
fn closest_typeof_value(input: &str) -> Option<&'static str> {
    let mut best: Option<(&str, usize)> = None;
    for &candidate in VALID_TYPEOF_VALUES {
        let dist = edit_distance(input, candidate);
        if let Some((_, best_dist)) = best {
            if dist < best_dist {
                best = Some((candidate, dist));
            }
        } else {
            best = Some((candidate, dist));
        }
    }
    // Only suggest if the edit distance is at most half the input length + 1
    best.and_then(|(c, d)| (d <= input.len().div_ceil(2)).then_some(c))
}

/// Simple Levenshtein edit distance.
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
fn edit_distance(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let m = a_bytes.len();
    let n = b_bytes.len();

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = usize::from(a_bytes[i - 1] != b_bytes[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ValidTypeof)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_valid_typeof() {
        for val in VALID_TYPEOF_VALUES {
            let source = format!(r#"if (typeof x === "{val}") {{}}"#);
            let diags = lint(&source);
            assert!(diags.is_empty(), "typeof === \"{val}\" should be valid");
        }
    }

    #[test]
    fn test_flags_invalid_typeof() {
        let diags = lint(r#"if (typeof x === "strig") {}"#);
        assert_eq!(diags.len(), 1, "typo 'strig' should be flagged");
    }

    #[test]
    fn test_flags_invalid_typeof_reversed() {
        let diags = lint(r#"if ("nubmer" === typeof x) {}"#);
        assert_eq!(
            diags.len(),
            1,
            "reversed comparison with typo should be flagged"
        );
    }

    #[test]
    fn test_flags_null_typeof() {
        let diags = lint(r#"if (typeof x === "null") {}"#);
        assert_eq!(diags.len(), 1, "'null' is not a valid typeof value");
    }

    #[test]
    fn test_allows_non_equality_operator() {
        let diags = lint(r#"const x = typeof y + "strig";"#);
        assert!(
            diags.is_empty(),
            "non-equality operator should not be checked"
        );
    }

    #[test]
    fn test_allows_no_string_literal() {
        let diags = lint("if (typeof x === y) {}");
        assert!(
            diags.is_empty(),
            "comparison against variable should not be checked"
        );
    }
}
