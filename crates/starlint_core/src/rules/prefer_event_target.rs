//! Rule: `prefer-event-target`
//!
//! Prefer `EventTarget` over Node.js `EventEmitter`. The `EventTarget` API
//! is a web standard available in browsers and modern Node.js, making code
//! more portable.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new EventEmitter()` and `extends EventEmitter`.
#[derive(Debug)]
pub struct PreferEventTarget;

impl NativeRule for PreferEventTarget {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-event-target".to_owned(),
            description: "Prefer `EventTarget` over `EventEmitter`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::NewExpression(new_expr) => {
                if matches!(
                    &new_expr.callee,
                    Expression::Identifier(id) if id.name.as_str() == "EventEmitter"
                ) {
                    ctx.report_warning(
                        "prefer-event-target",
                        "Prefer `EventTarget` over `EventEmitter`",
                        Span::new(new_expr.span.start, new_expr.span.end),
                    );
                }
            }
            AstKind::Class(class) => {
                let is_event_emitter = class.super_class.as_ref().is_some_and(|sc| {
                    matches!(sc, Expression::Identifier(id) if id.name.as_str() == "EventEmitter")
                });
                if is_event_emitter {
                    ctx.report_warning(
                        "prefer-event-target",
                        "Prefer extending `EventTarget` over `EventEmitter`",
                        Span::new(class.span.start, class.span.end),
                    );
                }
            }
            _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferEventTarget)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_event_emitter() {
        let diags = lint("var ee = new EventEmitter();");
        assert_eq!(diags.len(), 1, "new EventEmitter() should be flagged");
    }

    #[test]
    fn test_flags_extends_event_emitter() {
        let diags = lint("class Foo extends EventEmitter {}");
        assert_eq!(diags.len(), 1, "extends EventEmitter should be flagged");
    }

    #[test]
    fn test_allows_new_event_target() {
        let diags = lint("var et = new EventTarget();");
        assert!(diags.is_empty(), "new EventTarget() should not be flagged");
    }

    #[test]
    fn test_allows_extends_event_target() {
        let diags = lint("class Foo extends EventTarget {}");
        assert!(
            diags.is_empty(),
            "extends EventTarget should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_class() {
        let diags = lint("class Foo {}");
        assert!(diags.is_empty(), "plain class should not be flagged");
    }
}
