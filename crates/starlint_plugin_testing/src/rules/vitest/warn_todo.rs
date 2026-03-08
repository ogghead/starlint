//! Rule: `vitest/warn-todo`
//!
//! Warn when `test.todo` or `it.todo` is used. Todo tests are placeholders
//! for tests that need to be written. While useful during development, they
//! should not remain indefinitely in the test suite.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/warn-todo";

/// Warn when `test.todo` or `it.todo` is used.
#[derive(Debug)]
pub struct WarnTodo;

impl LintRule for WarnTodo {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when `test.todo` or `it.todo` is used".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `test.todo(...)` or `it.todo(...)`.
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "todo" {
            return;
        }

        let obj_name = match ctx.node(member.object) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
            _ => return,
        };

        if obj_name != "test" && obj_name != "it" {
            return;
        }

        // property is a String with no span. Use source text to find ".todo(" for the fix.
        let source = ctx.source_text();
        let call_start = usize::try_from(call.span.start).unwrap_or(0);
        let call_end = usize::try_from(call.span.end).unwrap_or(0);
        let call_text = source.get(call_start..call_end).unwrap_or("");
        let fix = call_text.find(".todo(").map(|offset| {
            let prop_start = call
                .span
                .start
                .saturating_add(offset as u32)
                .saturating_add(1);
            let prop_end = prop_start.saturating_add(4); // "todo" is 4 chars
            Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace `.todo` with `.skip`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(prop_start, prop_end),
                    replacement: "skip".to_owned(),
                }],
                is_snippet: false,
            }
        });

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: format!("`{obj_name}.todo` found — implement or remove this test placeholder"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(WarnTodo)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_test_todo() {
        let source = r#"test.todo("implement this");"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`test.todo` should be flagged");
    }

    #[test]
    fn test_flags_it_todo() {
        let source = r#"it.todo("implement this");"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`it.todo` should be flagged");
    }

    #[test]
    fn test_allows_regular_test() {
        let source = r#"test("my test", () => { expect(1).toBe(1); });"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "regular test without `.todo` should not be flagged"
        );
    }
}
