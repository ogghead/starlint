//! Rule: `no-useless-error-capture-stack-trace`
//!
//! Flag useless `Error.captureStackTrace(this, constructor)` calls. In modern
//! engines, `Error` subclasses automatically capture stack traces in the
//! constructor, making manual `captureStackTrace` calls redundant.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Error.captureStackTrace()` calls.
#[derive(Debug)]
pub struct NoUselessErrorCaptureStackTrace;

impl NativeRule for NoUselessErrorCaptureStackTrace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-error-capture-stack-trace".to_owned(),
            description: "Flag useless `Error.captureStackTrace()` calls".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "captureStackTrace" {
            return;
        }

        let is_error_object =
            matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Error");

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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessErrorCaptureStackTrace)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
