//! Rule: `jest/no-test-return-statement`
//!
//! Warn when a test callback has a return statement.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-test-return-statement";

/// Flags return statements inside test callbacks.
#[derive(Debug)]
pub struct NoTestReturnStatement;

impl LintRule for NoTestReturnStatement {
    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("test(") || source_text.contains("it("))
            && crate::is_test_file(file_path)
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

    fn run(&self, node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ReturnStatement(ret) = node else {
            return;
        };

        // Walk up the AST to check if inside a test callback
        if is_inside_test_via_ancestors(node_id, ctx) {
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

/// Walk up the AST parent chain to check if `node_id` is inside a
/// `test`/`it` callback. `O(depth)` instead of `O(source_length)`.
fn is_inside_test_via_ancestors(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let tree = ctx.tree();
    let mut current = tree.parent(node_id);
    while let Some(pid) = current {
        if let Some(AstNode::CallExpression(call)) = tree.get(pid) {
            if let Some(AstNode::IdentifierReference(id)) = tree.get(call.callee) {
                if id.name.as_str() == "test" || id.name.as_str() == "it" {
                    return true;
                }
            }
        }
        current = tree.parent(pid);
    }
    false
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

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
