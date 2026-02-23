//! Rule: `node/no-process-exit`
//!
//! Disallow the use of `process.exit()`. Calling `process.exit()` terminates
//! the process immediately without allowing cleanup handlers to run. Prefer
//! setting the exit code (`process.exitCode = 1`) and letting the process
//! exit naturally, or throwing an error.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags calls to `process.exit()`.
#[derive(Debug)]
pub struct NoProcessExit;

impl LintRule for NoProcessExit {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-process-exit".to_owned(),
            description: "Disallow the use of `process.exit()`".to_owned(),
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

        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "exit" {
            return;
        }

        let is_process = ctx.node(member.object).is_some_and(
            |n| matches!(n, AstNode::IdentifierReference(id) if id.name.as_str() == "process"),
        );

        if is_process {
            // Fix: process.exit(code) → process.exitCode = code
            let fix = call.arguments.first().map(|arg| {
                let source = ctx.source_text();
                let arg_span = ctx.node(*arg).map_or(Span::new(0, 0), |n| {
                    let s = n.span();
                    Span::new(s.start, s.end)
                });
                #[allow(clippy::as_conversions)]
                let code = source
                    .get(arg_span.start as usize..arg_span.end as usize)
                    .unwrap_or("1");
                let replacement = format!("process.exitCode = {code}");
                Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(call.span.start, call.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }
            });
            ctx.report(Diagnostic {
                rule_name: "node/no-process-exit".to_owned(),
                message: "Do not use `process.exit()` — set `process.exitCode` and allow the process to exit naturally".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoProcessExit)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_process_exit() {
        let diags = lint("process.exit(1);");
        assert_eq!(diags.len(), 1, "process.exit() should be flagged");
    }

    #[test]
    fn test_flags_process_exit_no_args() {
        let diags = lint("process.exit();");
        assert_eq!(
            diags.len(),
            1,
            "process.exit() without args should be flagged"
        );
    }

    #[test]
    fn test_allows_process_env() {
        let diags = lint("const e = process.env.NODE_ENV;");
        assert!(diags.is_empty(), "process.env should not be flagged");
    }

    #[test]
    fn test_allows_other_exit() {
        let diags = lint("app.exit();");
        assert!(diags.is_empty(), "non-process exit should not be flagged");
    }
}
