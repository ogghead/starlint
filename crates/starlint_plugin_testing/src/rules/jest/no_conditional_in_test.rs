//! Rule: `jest/no-conditional-in-test`
//!
//! Warn when if/switch/ternary is used inside test callbacks.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::ast_utils::is_inside_call_with_names;
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
    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("test(") || source_text.contains("it("))
            && crate::is_test_file(file_path)
    }

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

    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (stmt_type, span_start, span_end) = match node {
            AstNode::IfStatement(stmt) => ("if statement", stmt.span.start, stmt.span.end),
            AstNode::SwitchStatement(stmt) => ("switch statement", stmt.span.start, stmt.span.end),
            AstNode::ConditionalExpression(expr) => {
                ("ternary expression", expr.span.start, expr.span.end)
            }
            _ => return,
        };

        // Walk up the AST to check if inside a test callback
        if is_inside_call_with_names(node_id, ctx, &["test", "it"]) {
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

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoConditionalInTest);

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
