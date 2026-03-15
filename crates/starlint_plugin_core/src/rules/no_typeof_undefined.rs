//! Rule: `no-typeof-undefined`
//!
//! Prefer `x === undefined` over `typeof x === 'undefined'`. The `typeof`
//! guard is only needed for undeclared variables, which is rare in modern
//! module-based code.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `typeof x === 'undefined'` comparisons.
#[derive(Debug)]
pub struct NoTypeofUndefined;

impl LintRule for NoTypeofUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-typeof-undefined".to_owned(),
            description: "Prefer direct `undefined` comparison over `typeof` check".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BinaryExpression(expr) = node else {
            return;
        };

        // Only match equality/inequality operators.
        let new_op = match expr.operator {
            BinaryOperator::StrictEquality | BinaryOperator::Equality => "===",
            BinaryOperator::StrictInequality | BinaryOperator::Inequality => "!==",
            _ => return,
        };

        // Check both orderings: `typeof x === 'undefined'` and `'undefined' === typeof x`.
        let left = ctx.node(expr.left);
        let right = ctx.node(expr.right);

        let typeof_arg_id = match (left, right) {
            (Some(AstNode::UnaryExpression(unary)), Some(AstNode::StringLiteral(lit)))
                if unary.operator == UnaryOperator::Typeof && lit.value == "undefined" =>
            {
                Some(unary.argument)
            }
            (Some(AstNode::StringLiteral(lit)), Some(AstNode::UnaryExpression(unary)))
                if unary.operator == UnaryOperator::Typeof && lit.value == "undefined" =>
            {
                Some(unary.argument)
            }
            _ => None,
        };

        let Some(arg_id) = typeof_arg_id else {
            return;
        };

        let Some(typeof_arg_node) = ctx.node(arg_id) else {
            return;
        };
        let typeof_arg_span = typeof_arg_node.span();

        // Extract the argument text from source.
        let arg_start = usize::try_from(typeof_arg_span.start).unwrap_or(0);
        let arg_end = usize::try_from(typeof_arg_span.end).unwrap_or(0);
        let Some(arg_text) = ctx.source_text().get(arg_start..arg_end) else {
            return;
        };

        let replacement = format!("{arg_text} {new_op} undefined");

        ctx.report(Diagnostic {
            rule_name: "no-typeof-undefined".to_owned(),
            message: format!("Use `{replacement}` instead of `typeof` check"),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Direct `undefined` comparison is clearer in modern code".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(expr.span.start, expr.span.end),
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

    starlint_rule_framework::lint_rule_test!(NoTypeofUndefined);

    #[test]
    fn test_flags_typeof_strict_equals() {
        let diags = lint("if (typeof x === 'undefined') {}");
        assert_eq!(diags.len(), 1, "should flag typeof x === 'undefined'");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x === undefined"),
            "fix should replace with direct comparison"
        );
    }

    #[test]
    fn test_flags_typeof_strict_not_equals() {
        let diags = lint("if (typeof x !== 'undefined') {}");
        assert_eq!(diags.len(), 1, "should flag typeof x !== 'undefined'");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x !== undefined"),
            "fix should use !=="
        );
    }

    #[test]
    fn test_flags_reversed_order() {
        let diags = lint("if ('undefined' === typeof x) {}");
        assert_eq!(diags.len(), 1, "should flag reversed order");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x === undefined"),
            "fix should normalize to standard order"
        );
    }

    #[test]
    fn test_flags_loose_equals() {
        let diags = lint("if (typeof x == 'undefined') {}");
        assert_eq!(diags.len(), 1, "should flag loose equality");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("x === undefined"),
            "fix should upgrade to strict equality"
        );
    }

    #[test]
    fn test_flags_member_expression_arg() {
        let diags = lint("if (typeof obj.prop === 'undefined') {}");
        assert_eq!(diags.len(), 1, "should handle member expression arg");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("obj.prop === undefined"),
            "fix should preserve member expression"
        );
    }

    #[test]
    fn test_allows_typeof_string() {
        let diags = lint("if (typeof x === 'string') {}");
        assert!(
            diags.is_empty(),
            "typeof x === 'string' should not be flagged"
        );
    }

    #[test]
    fn test_allows_direct_undefined() {
        let diags = lint("if (x === undefined) {}");
        assert!(
            diags.is_empty(),
            "direct undefined comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_typeof_number() {
        let diags = lint("if (typeof x === 'number') {}");
        assert!(
            diags.is_empty(),
            "typeof x === 'number' should not be flagged"
        );
    }
}
