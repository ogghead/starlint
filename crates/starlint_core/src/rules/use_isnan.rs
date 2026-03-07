//! Rule: `use-isnan`
//!
//! Require `Number.isNaN()` instead of comparisons with `NaN`.
//! Because `NaN` is unique in JavaScript in that it is not equal to anything,
//! including itself, comparisons like `x === NaN` always evaluate to `false`
//! and `x !== NaN` always evaluates to `true`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags comparisons with `NaN` and suggests using `Number.isNaN()`.
#[derive(Debug)]
pub struct UseIsnan;

impl LintRule for UseIsnan {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "use-isnan".to_owned(),
            description: "Require `Number.isNaN()` instead of comparisons with `NaN`".to_owned(),
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

        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        let left_is_nan = is_nan(ctx, expr.left);
        let right_is_nan = is_nan(ctx, expr.right);

        if !left_is_nan && !right_is_nan {
            return;
        }

        // Determine the non-NaN operand to build the fix
        let non_nan_id = if left_is_nan { expr.right } else { expr.left };

        #[allow(clippy::as_conversions)]
        let fix = ctx.node(non_nan_id).and_then(|non_nan_node| {
            let non_nan_span = non_nan_node.span();
            ctx.source_text()
                .get(non_nan_span.start as usize..non_nan_span.end as usize)
                .map(|value_text| {
                    let is_negated = matches!(
                        expr.operator,
                        BinaryOperator::StrictInequality | BinaryOperator::Inequality
                    );
                    let replacement = if is_negated {
                        format!("!Number.isNaN({value_text})")
                    } else {
                        format!("Number.isNaN({value_text})")
                    };
                    Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(expr.span.start, expr.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                })
        });

        ctx.report(Diagnostic {
            rule_name: "use-isnan".to_owned(),
            message: "Comparisons with `NaN` always produce unexpected results".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Error,
            help: Some("Use `Number.isNaN(value)` instead".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

/// Check if a node is the identifier `NaN`.
fn is_nan(ctx: &LintContext<'_>, id: NodeId) -> bool {
    matches!(ctx.node(id), Some(AstNode::IdentifierReference(ident)) if ident.name == "NaN")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(UseIsnan)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_strict_equality_nan() {
        let diags = lint("if (x === NaN) {}");
        assert_eq!(diags.len(), 1, "=== NaN should be flagged");
    }

    #[test]
    fn test_flags_loose_equality_nan() {
        let diags = lint("if (x == NaN) {}");
        assert_eq!(diags.len(), 1, "== NaN should be flagged");
    }

    #[test]
    fn test_flags_inequality_nan() {
        let diags = lint("if (x !== NaN) {}");
        assert_eq!(diags.len(), 1, "!== NaN should be flagged");
    }

    #[test]
    fn test_flags_nan_on_left() {
        let diags = lint("if (NaN === x) {}");
        assert_eq!(diags.len(), 1, "NaN on left side should be flagged");
    }

    #[test]
    fn test_flags_less_than_nan() {
        let diags = lint("if (x < NaN) {}");
        assert_eq!(diags.len(), 1, "< NaN should be flagged");
    }

    #[test]
    fn test_allows_number_isnan() {
        let diags = lint("if (Number.isNaN(x)) {}");
        assert!(diags.is_empty(), "Number.isNaN() should not be flagged");
    }

    #[test]
    fn test_allows_isnan() {
        let diags = lint("if (isNaN(x)) {}");
        assert!(diags.is_empty(), "isNaN() should not be flagged");
    }

    #[test]
    fn test_allows_arithmetic_with_nan() {
        let diags = lint("const y = x + NaN;");
        assert!(
            diags.is_empty(),
            "arithmetic with NaN should not be flagged"
        );
    }
}
