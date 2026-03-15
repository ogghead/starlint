//! Rule: `no-process-exit` (unicorn)
//!
//! Disallow `process.exit()`. Prefer throwing an error or using
//! `process.exitCode` to allow cleanup and graceful shutdown.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `process.exit()` calls.
#[derive(Debug)]
pub struct NoProcessExit;

impl LintRule for NoProcessExit {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-process-exit".to_owned(),
            description: "Disallow `process.exit()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let is_process_exit = ctx.node(call.callee).is_some_and(|callee| {
            matches!(
                callee,
                AstNode::StaticMemberExpression(member)
                    if member.property == "exit"
                    && matches!(
                        ctx.node(member.object),
                        Some(AstNode::IdentifierReference(id)) if id.name == "process"
                    )
            )
        });

        if !is_process_exit {
            return;
        }

        // Fix: `process.exit(N)` → `process.exitCode = N`
        let fix = call.arguments.first().and_then(|&arg_id| {
            let arg_node = ctx.node(arg_id)?;
            let arg_span = arg_node.span();
            let source = ctx.source_text();
            let arg_text = source.get(arg_span.start as usize..arg_span.end as usize)?;
            Some(Fix {
                kind: FixKind::SuggestionFix,
                message: format!("Replace with `process.exitCode = {arg_text}`"),
                edits: vec![Edit {
                    span: Span::new(call.span.start, call.span.end),
                    replacement: format!("process.exitCode = {arg_text}"),
                }],
                is_snippet: false,
            })
        });

        ctx.report(Diagnostic {
            rule_name: "no-process-exit".to_owned(),
            message: "Avoid `process.exit()` — use `process.exitCode` or throw an error instead"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Set `process.exitCode` instead for graceful shutdown".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoProcessExit);

    #[test]
    fn test_flags_process_exit() {
        let diags = lint("process.exit(1);");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_process_exit_code() {
        let diags = lint("process.exitCode = 1;");
        assert!(diags.is_empty());
    }
}
