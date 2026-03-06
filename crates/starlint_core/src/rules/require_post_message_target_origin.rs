//! Rule: `require-post-message-target-origin`
//!
//! Require the `targetOrigin` argument in `postMessage()` calls. Omitting
//! the second argument means the message may be delivered to any origin,
//! which is a potential security risk. Always specify an explicit
//! `targetOrigin` such as a specific origin URL or `"*"`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `postMessage()` calls that are missing the `targetOrigin` argument.
#[derive(Debug)]
pub struct RequirePostMessageTargetOrigin;

impl NativeRule for RequirePostMessageTargetOrigin {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-post-message-target-origin".to_owned(),
            description: "Require `targetOrigin` argument in `postMessage()` calls".to_owned(),
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

        // Only match `<expr>.postMessage(...)` — member expression calls.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "postMessage" {
            return;
        }

        // The Web API signature is `postMessage(message, targetOrigin, [transfer])`.
        // If fewer than 2 arguments are provided, `targetOrigin` is missing.
        if call.arguments.len() < 2 {
            ctx.report(Diagnostic {
                rule_name: "require-post-message-target-origin".to_owned(),
                message: "`postMessage()` is missing the `targetOrigin` argument — this is a security risk".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequirePostMessageTargetOrigin)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_post_message_with_target_origin() {
        let diags = lint("window.postMessage('hi', '*');");
        assert!(
            diags.is_empty(),
            "postMessage with targetOrigin should not be flagged"
        );
    }

    #[test]
    fn test_flags_post_message_without_target_origin() {
        let diags = lint("window.postMessage('hi');");
        assert_eq!(
            diags.len(),
            1,
            "postMessage without targetOrigin should be flagged"
        );
    }

    #[test]
    fn test_flags_any_object_post_message() {
        let diags = lint("foo.postMessage('hi');");
        assert_eq!(
            diags.len(),
            1,
            "any object postMessage without targetOrigin should be flagged"
        );
    }

    #[test]
    fn test_allows_direct_post_message_with_args() {
        // Direct `postMessage('hi', '*')` is not a member expression call,
        // so this rule does not flag it.
        let diags = lint("postMessage('hi', '*');");
        assert!(
            diags.is_empty(),
            "direct postMessage call should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("window.addEventListener('message', handler);");
        assert!(
            diags.is_empty(),
            "unrelated method call should not be flagged"
        );
    }

    #[test]
    fn test_allows_post_message_with_three_args() {
        let diags = lint("worker.postMessage(data, '*', [buffer]);");
        assert!(
            diags.is_empty(),
            "postMessage with three arguments should not be flagged"
        );
    }

    #[test]
    fn test_flags_post_message_no_args() {
        let diags = lint("iframe.contentWindow.postMessage();");
        assert_eq!(
            diags.len(),
            1,
            "postMessage with no arguments should be flagged"
        );
    }
}
