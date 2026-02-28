//! Rule: `no-object-constructor`
//!
//! Disallow calls to the `Object` constructor without arguments.
//! Use `{}` instead of `new Object()` or `Object()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Object()` and `Object()` without arguments.
#[derive(Debug)]
pub struct NoObjectConstructor;

impl NativeRule for NoObjectConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-object-constructor".to_owned(),
            description: "Disallow `Object` constructor".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::NewExpression(new_expr) => {
                if matches!(&new_expr.callee, Expression::Identifier(id) if id.name.as_str() == "Object")
                    && new_expr.arguments.is_empty()
                {
                    ctx.report_warning(
                        "no-object-constructor",
                        "Disallow `Object` constructor — use `{}` instead",
                        Span::new(new_expr.span.start, new_expr.span.end),
                    );
                }
            }
            AstKind::CallExpression(call) => {
                if matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "Object")
                    && call.arguments.is_empty()
                {
                    ctx.report_warning(
                        "no-object-constructor",
                        "Disallow `Object` constructor — use `{}` instead",
                        Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoObjectConstructor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_object() {
        let diags = lint("var x = new Object();");
        assert_eq!(diags.len(), 1, "new Object() should be flagged");
    }

    #[test]
    fn test_flags_object_call() {
        let diags = lint("var x = Object();");
        assert_eq!(diags.len(), 1, "Object() should be flagged");
    }

    #[test]
    fn test_allows_object_literal() {
        let diags = lint("var x = {};");
        assert!(diags.is_empty(), "object literal should not be flagged");
    }

    #[test]
    fn test_allows_object_with_args() {
        let diags = lint("var x = Object(value);");
        assert!(diags.is_empty(), "Object() with args should not be flagged");
    }
}
