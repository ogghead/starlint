//! Rule: `no-invalid-remove-event-listener`
//!
//! Flag `removeEventListener` calls where the listener argument is an
//! inline function expression or arrow function. Inline functions create
//! a new reference each time, so they can never match a previously added
//! listener — making the `removeEventListener` call a no-op bug.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `removeEventListener` calls with inline function listeners.
#[derive(Debug)]
pub struct NoInvalidRemoveEventListener;

impl NativeRule for NoInvalidRemoveEventListener {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-invalid-remove-event-listener".to_owned(),
            description: "Disallow inline function listeners in `removeEventListener` calls"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `.removeEventListener(...)` or `removeEventListener(...)`
        if !is_remove_event_listener_call(&call.callee) {
            return;
        }

        // The listener is the second argument
        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };

        // Flag if the listener is an inline function or arrow function
        if is_inline_function(second_arg) {
            ctx.report_error(
                "no-invalid-remove-event-listener",
                "Inline function passed to `removeEventListener` will never match a previously added listener",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if a call expression's callee is `removeEventListener` (either
/// as a member property or a direct identifier).
fn is_remove_event_listener_call(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::StaticMemberExpression(member) => {
            member.property.name.as_str() == "removeEventListener"
        }
        Expression::Identifier(id) => id.name.as_str() == "removeEventListener",
        _ => false,
    }
}

/// Check if an argument is an inline function expression or arrow function.
const fn is_inline_function(arg: &Argument<'_>) -> bool {
    matches!(
        arg,
        Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_)
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInvalidRemoveEventListener)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_arrow_function_listener() {
        let diags = lint("el.removeEventListener('click', () => {});");
        assert_eq!(diags.len(), 1, "inline arrow listener should be flagged");
    }

    #[test]
    fn test_flags_function_expression_listener() {
        let diags = lint("el.removeEventListener('click', function() {});");
        assert_eq!(
            diags.len(),
            1,
            "inline function expression listener should be flagged"
        );
    }

    #[test]
    fn test_flags_arrow_with_body() {
        let diags = lint("el.removeEventListener('click', (e) => { console.log(e); });");
        assert_eq!(diags.len(), 1, "inline arrow with body should be flagged");
    }

    #[test]
    fn test_allows_named_handler() {
        let diags = lint("el.removeEventListener('click', handler);");
        assert!(
            diags.is_empty(),
            "named handler reference should not be flagged"
        );
    }

    #[test]
    fn test_allows_method_reference() {
        let diags = lint("el.removeEventListener('click', this.handleClick);");
        assert!(diags.is_empty(), "method reference should not be flagged");
    }

    #[test]
    fn test_allows_add_event_listener_inline() {
        let diags = lint("el.addEventListener('click', () => {});");
        assert!(
            diags.is_empty(),
            "addEventListener with inline function should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_listener_arg() {
        let diags = lint("el.removeEventListener('click');");
        assert!(
            diags.is_empty(),
            "removeEventListener with only one arg should not be flagged"
        );
    }
}
