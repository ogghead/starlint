//! Rule: `no-alert`
//!
//! Disallow the use of `alert`, `confirm`, and `prompt`. These are
//! browser-native dialogs that are generally bad UX and should not
//! appear in production code.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `alert()`, `confirm()`, and `prompt()` calls.
#[derive(Debug)]
pub struct NoAlert;

/// Blocked global function names.
const BLOCKED_NAMES: &[&str] = &["alert", "confirm", "prompt"];

impl NativeRule for NoAlert {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-alert".to_owned(),
            description: "Disallow the use of `alert`, `confirm`, and `prompt`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_blocked = match &call.callee {
            Expression::Identifier(id) => BLOCKED_NAMES.contains(&id.name.as_str()),
            Expression::StaticMemberExpression(member) => {
                BLOCKED_NAMES.contains(&member.property.name.as_str())
                    && matches!(
                        &member.object,
                        Expression::Identifier(id) if id.name.as_str() == "window"
                            || id.name.as_str() == "globalThis"
                    )
            }
            _ => false,
        };

        if is_blocked {
            ctx.report_warning(
                "no-alert",
                "Unexpected `alert`, `confirm`, or `prompt`",
                Span::new(call.span.start, call.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAlert)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_alert() {
        let diags = lint("alert('hello');");
        assert_eq!(diags.len(), 1, "alert() should be flagged");
    }

    #[test]
    fn test_flags_confirm() {
        let diags = lint("confirm('sure?');");
        assert_eq!(diags.len(), 1, "confirm() should be flagged");
    }

    #[test]
    fn test_flags_prompt() {
        let diags = lint("prompt('name?');");
        assert_eq!(diags.len(), 1, "prompt() should be flagged");
    }

    #[test]
    fn test_flags_window_alert() {
        let diags = lint("window.alert('hello');");
        assert_eq!(diags.len(), 1, "window.alert() should be flagged");
    }

    #[test]
    fn test_allows_normal_function() {
        let diags = lint("doSomething();");
        assert!(diags.is_empty(), "normal function should not be flagged");
    }
}
