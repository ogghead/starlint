//! Rule: `number-arg-out-of-range` (OXC)
//!
//! Detect calls to functions like `parseInt(x, radix)` or `Number.toFixed(n)`
//! where the numeric argument is outside the valid range.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags numeric arguments outside valid ranges.
#[derive(Debug)]
pub struct NumberArgOutOfRange;

impl LintRule for NumberArgOutOfRange {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "number-arg-out-of-range".to_owned(),
            description: "Detect numeric arguments outside valid ranges".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let findings = check_parse_int(ctx, call)
            .or_else(|| check_method_range(ctx, call, "toFixed", 0, 100))
            .or_else(|| check_method_range(ctx, call, "toPrecision", 1, 100))
            .or_else(|| check_method_range(ctx, call, "toExponential", 0, 100));

        if let Some(message) = findings {
            ctx.report(Diagnostic {
                rule_name: "number-arg-out-of-range".to_owned(),
                message,
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check parseInt radix (must be 2-36).
fn check_parse_int(
    ctx: &LintContext<'_>,
    call: &starlint_ast::node::CallExpressionNode,
) -> Option<String> {
    let callee_node = ctx.node(call.callee)?;
    let is_parse_int = match callee_node {
        AstNode::IdentifierReference(id) => id.name.as_str() == "parseInt",
        AstNode::StaticMemberExpression(member) => member.property.as_str() == "parseInt"
            && ctx.node(member.object).is_some_and(
                |n| matches!(n, AstNode::IdentifierReference(id) if id.name.as_str() == "Number"),
            ),
        _ => false,
    };

    if !is_parse_int {
        return None;
    }

    let radix_arg_id = *call.arguments.get(1)?;
    let radix_node = ctx.node(radix_arg_id)?;
    let radix = get_integer_value(radix_node)?;

    if !(2..=36).contains(&radix) {
        return Some(format!(
            "`parseInt()` radix must be between 2 and 36, got {radix}"
        ));
    }

    None
}

/// Check a method call's first argument against a valid range.
fn check_method_range(
    ctx: &LintContext<'_>,
    call: &starlint_ast::node::CallExpressionNode,
    method_name: &str,
    min: i64,
    max: i64,
) -> Option<String> {
    let callee_node = ctx.node(call.callee)?;
    let is_method = matches!(
        callee_node,
        AstNode::StaticMemberExpression(member) if member.property.as_str() == method_name
    );

    if !is_method {
        return None;
    }

    let arg_id = *call.arguments.first()?;
    let arg_node = ctx.node(arg_id)?;
    let value = get_integer_value(arg_node)?;

    if !(min..=max).contains(&value) {
        return Some(format!(
            "`.{method_name}()` argument must be between {min} and {max}, got {value}"
        ));
    }

    None
}

/// Get integer value from a numeric literal `AstNode`.
///
/// Returns `None` if the node is not a numeric literal or if the value
/// cannot be safely converted to i64.
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
fn get_integer_value(node: &AstNode) -> Option<i64> {
    match node {
        AstNode::NumericLiteral(n) => {
            let val = n.value;
            // Only consider integer values (no fractional part) within i64 range
            #[allow(clippy::float_cmp)]
            let is_integer = val == val.trunc() && val >= i64::MIN as f64 && val <= i64::MAX as f64;
            is_integer.then_some(val as i64)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NumberArgOutOfRange)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_parse_int_bad_radix() {
        let diags = lint("parseInt('10', 37);");
        assert_eq!(diags.len(), 1, "parseInt with radix 37 should be flagged");
    }

    #[test]
    fn test_flags_parse_int_radix_zero() {
        let diags = lint("parseInt('10', 0);");
        assert_eq!(diags.len(), 1, "parseInt with radix 0 should be flagged");
    }

    #[test]
    fn test_allows_parse_int_valid_radix() {
        let diags = lint("parseInt('10', 16);");
        assert!(
            diags.is_empty(),
            "parseInt with radix 16 should not be flagged"
        );
    }

    #[test]
    fn test_flags_to_fixed_out_of_range() {
        let diags = lint("n.toFixed(101);");
        assert_eq!(diags.len(), 1, "toFixed(101) should be flagged");
    }

    #[test]
    fn test_allows_to_fixed_valid() {
        let diags = lint("n.toFixed(2);");
        assert!(diags.is_empty(), "toFixed(2) should not be flagged");
    }
}
