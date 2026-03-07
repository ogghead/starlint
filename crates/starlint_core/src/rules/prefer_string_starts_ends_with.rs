//! Rule: `prefer-string-starts-ends-with` (unicorn)
//!
//! Prefer `String#startsWith()` and `String#endsWith()` over regex tests
//! or manual index checks. For example, `/^foo/.test(str)` should be
//! `str.startsWith('foo')` and `str.indexOf('x') === 0` should be
//! `str.startsWith('x')`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::BinaryOperator;
use starlint_ast::types::NodeId;

/// Flags patterns that can use `startsWith`/`endsWith`.
#[derive(Debug)]
pub struct PreferStringStartsEndsWith;

impl LintRule for PreferStringStartsEndsWith {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-starts-ends-with".to_owned(),
            description: "Prefer `startsWith()` and `endsWith()` over alternatives".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BinaryExpression, AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::BinaryExpression(expr) => check_index_of_comparison(expr, ctx),
            AstNode::CallExpression(call) => check_regex_test(call, ctx),
            _ => {}
        }
    }
}

/// Check for `.indexOf(x) === 0` pattern.
#[allow(clippy::as_conversions)]
fn check_index_of_comparison(
    expr: &starlint_ast::node::BinaryExpressionNode,
    ctx: &mut LintContext<'_>,
) {
    // Match: str.indexOf(x) === 0 or 0 === str.indexOf(x)
    // We need to find which side is the call expression and which is the zero literal.
    let (call_id, _zero_id) = {
        let left_is_call = matches!(ctx.node(expr.left), Some(AstNode::CallExpression(_)));
        let right_is_call = matches!(ctx.node(expr.right), Some(AstNode::CallExpression(_)));
        let left_is_zero = is_zero_literal(expr.left, ctx);
        let right_is_zero = is_zero_literal(expr.right, ctx);

        if left_is_call && right_is_zero {
            (expr.left, expr.right)
        } else if right_is_call && left_is_zero {
            (expr.right, expr.left)
        } else {
            return;
        }
    };

    if !matches!(
        expr.operator,
        BinaryOperator::StrictEquality | BinaryOperator::Equality
    ) {
        return;
    }

    let Some(AstNode::CallExpression(call)) = ctx.node(call_id) else {
        return;
    };

    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return;
    };

    if member.property.as_str() != "indexOf" {
        return;
    }

    if call.arguments.len() != 1 {
        return;
    }

    // Build fix: `obj.startsWith(arg)`
    let source = ctx.source_text();
    let obj_span = ctx.node(member.object).map(starlint_ast::AstNode::span);
    let (obj_start, obj_end) = match obj_span {
        Some(s) => (s.start as usize, s.end as usize),
        None => return,
    };
    let obj_text = source.get(obj_start..obj_end).unwrap_or("");

    let fix = call.arguments.first().and_then(|&arg_id| {
        let arg_span = ctx.node(arg_id)?.span();
        let a_start = arg_span.start as usize;
        let a_end = arg_span.end as usize;
        let arg_text = source.get(a_start..a_end)?;
        Some(Fix {
            kind: FixKind::SuggestionFix,
            message: "Replace with `.startsWith()`".to_owned(),
            edits: vec![Edit {
                span: Span::new(expr.span.start, expr.span.end),
                replacement: format!("{obj_text}.startsWith({arg_text})"),
            }],
            is_snippet: false,
        })
    });

    ctx.report(Diagnostic {
        rule_name: "prefer-string-starts-ends-with".to_owned(),
        message: "Prefer `startsWith()` over `.indexOf() === 0`".to_owned(),
        span: Span::new(expr.span.start, expr.span.end),
        severity: Severity::Warning,
        help: Some("Replace with `.startsWith()`".to_owned()),
        fix,
        labels: vec![],
    });
}

/// Check for `/^foo/.test(str)` or `/foo$/.test(str)` pattern.
#[allow(clippy::as_conversions)]
fn check_regex_test(call: &starlint_ast::node::CallExpressionNode, ctx: &mut LintContext<'_>) {
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
        return;
    };

    if member.property.as_str() != "test" {
        return;
    }

    let Some(AstNode::RegExpLiteral(regex)) = ctx.node(member.object) else {
        return;
    };

    let pattern = regex.pattern.as_str();

    let (kind, literal_part) = if let Some(rest) = pattern.strip_prefix('^') {
        ("startsWith", rest)
    } else if let Some(rest) = pattern.strip_suffix('$') {
        ("endsWith", rest)
    } else {
        return;
    };

    // Only flag if the literal part is a simple string (no regex metacharacters)
    if literal_part.chars().any(|c| {
        matches!(
            c,
            '.' | '*' | '+' | '?' | '[' | ']' | '(' | ')' | '{' | '}' | '|' | '\\'
        )
    }) {
        return;
    }

    if literal_part.is_empty() {
        return;
    }

    // Build fix: `str.startsWith('literal')` or `str.endsWith('literal')`
    let source = ctx.source_text();
    let fix = call.arguments.first().and_then(|&arg_id| {
        let arg_span = ctx.node(arg_id)?.span();
        let a_start = arg_span.start as usize;
        let a_end = arg_span.end as usize;
        let arg_text = source.get(a_start..a_end)?;
        Some(Fix {
            kind: FixKind::SuggestionFix,
            message: format!("Replace with `.{kind}('{literal_part}')` "),
            edits: vec![Edit {
                span: Span::new(call.span.start, call.span.end),
                replacement: format!("{arg_text}.{kind}('{literal_part}')"),
            }],
            is_snippet: false,
        })
    });

    ctx.report(Diagnostic {
        rule_name: "prefer-string-starts-ends-with".to_owned(),
        message: format!("Prefer `.{kind}('{literal_part}')` over regex test"),
        span: Span::new(call.span.start, call.span.end),
        severity: Severity::Warning,
        help: Some(format!("Replace with `.{kind}('{literal_part}')`")),
        fix,
        labels: vec![],
    });
}

/// Check if a node is the numeric literal `0`.
fn is_zero_literal(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    if let Some(AstNode::NumericLiteral(lit)) = ctx.node(node_id) {
        return lit.value.abs() < f64::EPSILON;
    }
    false
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferStringStartsEndsWith)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_index_of_equals_zero() {
        let diags = lint("if (str.indexOf('foo') === 0) {}");
        assert_eq!(diags.len(), 1, "indexOf === 0 should be flagged");
    }

    #[test]
    fn test_flags_regex_starts_with() {
        let diags = lint("if (/^foo/.test(str)) {}");
        assert_eq!(diags.len(), 1, "/^foo/.test should be flagged");
    }

    #[test]
    fn test_flags_regex_ends_with() {
        let diags = lint("if (/bar$/.test(str)) {}");
        assert_eq!(diags.len(), 1, "/bar$/.test should be flagged");
    }

    #[test]
    fn test_allows_starts_with() {
        let diags = lint("if (str.startsWith('foo')) {}");
        assert!(diags.is_empty(), "startsWith should not be flagged");
    }

    #[test]
    fn test_allows_ends_with() {
        let diags = lint("if (str.endsWith('bar')) {}");
        assert!(diags.is_empty(), "endsWith should not be flagged");
    }

    #[test]
    fn test_allows_complex_regex() {
        let diags = lint("if (/^foo.*bar/.test(str)) {}");
        assert!(diags.is_empty(), "complex regex should not be flagged");
    }

    #[test]
    fn test_allows_index_of_not_zero() {
        let diags = lint("if (str.indexOf('foo') === 3) {}");
        assert!(
            diags.is_empty(),
            "indexOf !== 0 comparison should not be flagged"
        );
    }
}
