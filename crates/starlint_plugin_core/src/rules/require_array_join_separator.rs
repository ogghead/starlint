//! Rule: `require-array-join-separator` (unicorn)
//!
//! Enforce using the separator argument with `Array#join()`.
//! Calling `.join()` without arguments uses `","` as a default separator,
//! which is often not the intent. Require an explicit separator.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `Array#join()` calls without an explicit separator argument.
#[derive(Debug)]
pub struct RequireArrayJoinSeparator;

impl LintRule for RequireArrayJoinSeparator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-array-join-separator".to_owned(),
            description: "Enforce using the separator argument with `Array#join()`".to_owned(),
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

        let is_join = matches!(
            ctx.node(call.callee),
            Some(AstNode::StaticMemberExpression(member)) if member.property.as_str() == "join"
        );

        if !is_join {
            return;
        }

        // Flag if no arguments provided
        if call.arguments.is_empty() {
            ctx.report(Diagnostic {
                rule_name: "require-array-join-separator".to_owned(),
                message:
                    "Missing separator argument in `.join()` — the default `\",\"` may not be intended"
                        .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Add explicit separator argument".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `\",\"` separator".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(
                            call.span.end.saturating_sub(1),
                            call.span.end.saturating_sub(1),
                        ),
                        replacement: "\",\"".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireArrayJoinSeparator)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_join_without_args() {
        let diags = lint("[1, 2, 3].join();");
        assert_eq!(diags.len(), 1, "join() without args should be flagged");
    }

    #[test]
    fn test_allows_join_with_separator() {
        let diags = lint("[1, 2, 3].join(', ');");
        assert!(
            diags.is_empty(),
            "join with separator should not be flagged"
        );
    }

    #[test]
    fn test_allows_join_with_empty_string() {
        let diags = lint("[1, 2, 3].join('');");
        assert!(
            diags.is_empty(),
            "join with empty string should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_methods() {
        let diags = lint("[1, 2, 3].map(x => x);");
        assert!(diags.is_empty(), "other methods should not be flagged");
    }
}
