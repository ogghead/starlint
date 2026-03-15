//! Rule: `no-empty-function`
//!
//! Disallow empty function bodies. Empty functions are often indicators
//! of missing implementation or leftover stubs.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags functions with empty bodies.
#[derive(Debug)]
pub struct NoEmptyFunction;

impl LintRule for NoEmptyFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-function".to_owned(),
            description: "Disallow empty function bodies".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::FunctionBody])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::FunctionBody(body) = node else {
            return;
        };

        // Empty body: no statements
        if body.statements.is_empty() {
            // Check if there are any comments inside the function body by
            // looking at the raw source. If the body contains a comment,
            // it's intentionally empty (placeholder).
            let source = ctx.source_text();
            let start = usize::try_from(body.span.start).unwrap_or(0);
            let end = usize::try_from(body.span.end).unwrap_or(0);
            let body_text = source.get(start..end).unwrap_or("");
            let has_comment = body_text.contains("//") || body_text.contains("/*");
            let span_start = body.span.start;
            let span_end = body.span.end;

            if !has_comment {
                // Fix: insert a placeholder comment inside the empty body
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `/* empty */` comment".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(span_start, span_end),
                        replacement: "{ /* empty */ }".to_owned(),
                    }],
                    is_snippet: false,
                });

                ctx.report(Diagnostic {
                    rule_name: "no-empty-function".to_owned(),
                    message: "Unexpected empty function body".to_owned(),
                    span: Span::new(span_start, span_end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoEmptyFunction);

    #[test]
    fn test_flags_empty_function() {
        let diags = lint("function foo() {}");
        assert_eq!(diags.len(), 1, "empty function should be flagged");
    }

    #[test]
    fn test_flags_empty_arrow() {
        let diags = lint("var f = () => {};");
        assert_eq!(diags.len(), 1, "empty arrow function should be flagged");
    }

    #[test]
    fn test_allows_function_with_body() {
        let diags = lint("function foo() { return 1; }");
        assert!(diags.is_empty(), "function with body should not be flagged");
    }

    #[test]
    fn test_allows_function_with_comment() {
        let diags = lint("function foo() { /* intentionally empty */ }");
        assert!(
            diags.is_empty(),
            "function with comment should not be flagged"
        );
    }
}
