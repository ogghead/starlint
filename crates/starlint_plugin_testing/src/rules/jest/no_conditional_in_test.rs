//! Rule: `jest/no-conditional-in-test`
//!
//! Warn when if/switch/ternary is used inside test callbacks.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-conditional-in-test";

/// Flags conditional statements inside test callbacks.
///
/// Simplified: detects `if`/`switch`/ternary at the source level within test
/// file context. The rule matches `AstNode::IfStatement`, `AstNode::SwitchStatement`,
/// and `AstNode::ConditionalExpression` and checks if they appear inside a
/// `test`/`it` callback by scanning the preceding source.
#[derive(Debug)]
pub struct NoConditionalInTest;

impl LintRule for NoConditionalInTest {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow conditional logic in tests".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ConditionalExpression,
            AstNodeType::IfStatement,
            AstNodeType::SwitchStatement,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (stmt_type, span_start, span_end) = match node {
            AstNode::IfStatement(stmt) => ("if statement", stmt.span.start, stmt.span.end),
            AstNode::SwitchStatement(stmt) => ("switch statement", stmt.span.start, stmt.span.end),
            AstNode::ConditionalExpression(expr) => {
                ("ternary expression", expr.span.start, expr.span.end)
            }
            _ => return,
        };

        // Check if this conditional is inside a test callback by scanning the source
        // before it for `test(` or `it(` patterns
        let source = ctx.source_text();
        let pos = usize::try_from(span_start).unwrap_or(0);
        let before = source.get(..pos).unwrap_or("");

        if is_inside_test_callback(before) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Unexpected {stmt_type} inside a test — tests should not contain conditional logic"),
                span: Span::new(span_start, span_end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if a position is inside a test/it callback by counting open/close braces
/// after the last `test(` or `it(` call.
fn is_inside_test_callback(before: &str) -> bool {
    // Find the last occurrence of `test(` or `it(`
    let last_test = before.rfind("test(");
    let last_it = before.rfind("it(");

    let call_pos = match (last_test, last_it) {
        (Some(t), Some(i)) => Some(t.max(i)),
        (Some(t), None) => Some(t),
        (None, Some(i)) => Some(i),
        (None, None) => None,
    };

    let Some(pos) = call_pos else {
        return false;
    };

    // Count braces from the call position to see if we're still inside the callback
    let after_call = before.get(pos..).unwrap_or("");
    let mut brace_depth: i32 = 0;
    for ch in after_call.chars() {
        if ch == '{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
        }
    }

    // If braces are still open, we're inside the callback
    brace_depth > 0
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConditionalInTest)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_if_in_test() {
        let source = "test('cond', () => { if (true) { console.log('x'); } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "if inside test should be flagged");
    }

    #[test]
    fn test_flags_ternary_in_test() {
        let source = "it('cond', () => { const x = true ? 1 : 2; });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "ternary inside test should be flagged");
    }

    #[test]
    fn test_allows_if_outside_test() {
        let source = "if (process.env.CI) { console.log('ci'); }";
        let diags = lint(source);
        assert!(diags.is_empty(), "if outside test should not be flagged");
    }
}
