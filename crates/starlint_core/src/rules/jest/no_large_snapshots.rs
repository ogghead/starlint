//! Rule: `jest/no-large-snapshots`
//!
//! Warn when inline snapshot strings are too long (> 50 lines).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/no-large-snapshots";

/// Maximum number of lines allowed in an inline snapshot.
const MAX_LINES: usize = 50;

/// Flags inline snapshot arguments that exceed the line threshold.
#[derive(Debug)]
pub struct NoLargeSnapshots;

impl LintRule for NoLargeSnapshots {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow large inline snapshots".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `.toMatchInlineSnapshot(...)` pattern
        let is_inline_snapshot = match ctx.node(call.callee) {
            Some(AstNode::StaticMemberExpression(member)) => {
                member.property.as_str() == "toMatchInlineSnapshot"
            }
            _ => false,
        };

        if !is_inline_snapshot {
            return;
        }

        // Check the first argument — should be a string literal or template literal
        let Some(first_arg_id) = call.arguments.first() else {
            return;
        };

        let line_count = match ctx.node(*first_arg_id) {
            Some(AstNode::StringLiteral(s)) => count_lines(s.value.as_str()),
            Some(AstNode::TemplateLiteral(t)) => {
                // Count lines across all quasis (template string parts)
                // In starlint_ast, quasis are Box<[String]> directly
                let mut lines: usize = 0;
                for quasi in &t.quasis {
                    lines = lines.saturating_add(count_lines(quasi.as_str()));
                }
                lines
            }
            _ => return,
        };

        if line_count > MAX_LINES {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Inline snapshot is too large ({line_count} lines, max {MAX_LINES}) — use an external snapshot file instead"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Count the number of lines in a string.
fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        return 1;
    }
    s.lines().count().max(1)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLargeSnapshots)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_large_inline_snapshot() {
        // Generate a string with 60 lines
        let big_snapshot: String = (0..60)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\\n");
        let source = format!("expect(result).toMatchInlineSnapshot(\"{big_snapshot}\");");
        let diags = lint(&source);
        assert_eq!(diags.len(), 1, "large inline snapshot should be flagged");
    }

    #[test]
    fn test_allows_small_inline_snapshot() {
        let source = r#"expect(result).toMatchInlineSnapshot("small value");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "small inline snapshot should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_snapshot_call() {
        let diags = lint("expect(result).toBe(true);");
        assert!(
            diags.is_empty(),
            "non-snapshot matcher should not be flagged"
        );
    }
}
