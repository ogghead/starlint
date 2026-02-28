//! Rule: `no-new-wrappers`
//!
//! Disallow `new String()`, `new Number()`, `new Boolean()`.
//! Using primitive wrapper constructors creates objects, not primitives.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new` on primitive wrapper constructors.
#[derive(Debug)]
pub struct NoNewWrappers;

/// Primitive wrapper types that should not be used with `new`.
const WRAPPER_TYPES: &[&str] = &["String", "Number", "Boolean"];

impl NativeRule for NoNewWrappers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new-wrappers".to_owned(),
            description: "Disallow primitive wrapper constructors".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        if let Expression::Identifier(id) = &new_expr.callee {
            let name = id.name.as_str();
            if WRAPPER_TYPES.contains(&name) {
                ctx.report_warning(
                    "no-new-wrappers",
                    &format!("Do not use `new {name}()` — use the primitive instead"),
                    Span::new(new_expr.span.start, new_expr.span.end),
                );
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNewWrappers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_string() {
        let diags = lint("var s = new String('hello');");
        assert_eq!(diags.len(), 1, "new String() should be flagged");
    }

    #[test]
    fn test_flags_new_number() {
        let diags = lint("var n = new Number(42);");
        assert_eq!(diags.len(), 1, "new Number() should be flagged");
    }

    #[test]
    fn test_flags_new_boolean() {
        let diags = lint("var b = new Boolean(true);");
        assert_eq!(diags.len(), 1, "new Boolean() should be flagged");
    }

    #[test]
    fn test_allows_string_function() {
        let diags = lint("var s = String(42);");
        assert!(
            diags.is_empty(),
            "String() without new should not be flagged"
        );
    }
}
