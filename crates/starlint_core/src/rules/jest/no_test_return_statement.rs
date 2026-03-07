//! Rule: `jest/no-test-return-statement`
//!
//! Warn when a test callback has a return statement.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/no-test-return-statement";

/// Flags return statements inside test callbacks.
#[derive(Debug)]
pub struct NoTestReturnStatement;

impl LintRule for NoTestReturnStatement {
    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("test(") || source_text.contains("it(")
    }

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow return statements in test callbacks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ReturnStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ReturnStatement(ret) = node else {
            return;
        };

        // Check if this return is inside a test/it callback
        let source = ctx.source_text();
        let pos = usize::try_from(ret.span.start).unwrap_or(0);
        let before = source.get(..pos).unwrap_or("");

        if is_inside_test_callback(before) {
            // Build fix: replace `return <expr>;` with `return;`
            let fix = ret.argument.as_ref().map(|_| Fix {
                kind: FixKind::SuggestionFix,
                message: "Remove return value".to_owned(),
                edits: vec![Edit {
                    span: Span::new(ret.span.start, ret.span.end),
                    replacement: "return;".to_owned(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unexpected return statement in test — tests should not return values"
                    .to_owned(),
                span: Span::new(ret.span.start, ret.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a position is inside a test/it callback by counting brace depth.
fn is_inside_test_callback(before: &str) -> bool {
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

    let after_call = before.get(pos..).unwrap_or("");
    let mut brace_depth: i32 = 0;
    for ch in after_call.chars() {
        if ch == '{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
        }
    }

    brace_depth > 0
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoTestReturnStatement)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_return_in_test() {
        let source = "test('returns', () => { return 42; });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "return in test callback should be flagged");
    }

    #[test]
    fn test_allows_test_without_return() {
        let source = "test('ok', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_outside_test() {
        let source = "function helper() { return 1; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "return outside test should not be flagged"
        );
    }
}
