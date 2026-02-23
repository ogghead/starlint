//! Rule: `no-useless-error-capture-stack-trace`
//!
//! Flag useless `Error.captureStackTrace(this, constructor)` calls. In modern
//! engines, `Error` subclasses automatically capture stack traces in the
//! constructor, making manual `captureStackTrace` calls redundant.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `Error.captureStackTrace()` calls.
#[derive(Debug)]
pub struct NoUselessErrorCaptureStackTrace;

impl LintRule for NoUselessErrorCaptureStackTrace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-error-capture-stack-trace".to_owned(),
            description: "Flag useless `Error.captureStackTrace()` calls".to_owned(),
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

        if member.property.as_str() != "captureStackTrace" {
            return;
        }

        let is_error_object = matches!(ctx.node(member.object), Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "Error");

        if is_error_object {
            let call_span = Span::new(call.span.start, call.span.end);
            // Extend span to include trailing semicolon if present
            let source = ctx.source_text();
            let end = usize::try_from(call.span.end).unwrap_or(0);
            let fix_end = if source.as_bytes().get(end) == Some(&b';') {
                call.span.end.saturating_add(1)
            } else {
                call.span.end
            };
            let fix_span = Span::new(call.span.start, fix_end);
            ctx.report(Diagnostic {
                rule_name: "no-useless-error-capture-stack-trace".to_owned(),
                message: "`Error.captureStackTrace()` is redundant — `Error` subclasses automatically capture stack traces".to_owned(),
                span: call_span,
                severity: Severity::Warning,
                help: Some("Remove the `Error.captureStackTrace()` call".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove `Error.captureStackTrace()` call".to_owned(),
                    edits: vec![Edit {
                        span: fix_span,
                        replacement: String::new(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessErrorCaptureStackTrace)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_error_capture_stack_trace() {
        let diags = lint("Error.captureStackTrace(this, MyError);");
        assert_eq!(
            diags.len(),
            1,
            "Error.captureStackTrace() should be flagged"
        );
    }

    #[test]
    fn test_flags_error_capture_stack_trace_single_arg() {
        let diags = lint("Error.captureStackTrace(this);");
        assert_eq!(
            diags.len(),
            1,
            "Error.captureStackTrace() with one arg should be flagged"
        );
    }

    #[test]
    fn test_allows_new_error() {
        let diags = lint("new Error('msg');");
        assert!(diags.is_empty(), "new Error() should not be flagged");
    }

    #[test]
    fn test_allows_non_call_reference() {
        let diags = lint("console.log(Error.captureStackTrace);");
        assert!(
            diags.is_empty(),
            "reference without call should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_capture_stack_trace() {
        let diags = lint("CustomError.captureStackTrace(this);");
        assert!(
            diags.is_empty(),
            "captureStackTrace on non-Error object should not be flagged"
        );
    }

    #[test]
    fn test_allows_error_other_method() {
        let diags = lint("Error.isError(obj);");
        assert!(
            diags.is_empty(),
            "Error with a different method should not be flagged"
        );
    }
}
