//! Rule: `prefer-reflect-apply`
//!
//! Prefer `Reflect.apply()` over `Function.prototype.apply()`. The
//! `Reflect.apply()` method is clearer and avoids relying on `.apply()`
//! being present on the function object.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.apply()` calls with two arguments, suggesting `Reflect.apply()`.
#[derive(Debug)]
pub struct PreferReflectApply;

impl NativeRule for PreferReflectApply {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-reflect-apply".to_owned(),
            description: "Prefer `Reflect.apply()` over `Function.prototype.apply()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
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

        // Must be calling `.apply()`
        if member.property.name.as_str() != "apply" {
            return;
        }

        // Must have exactly 2 arguments (thisArg, argsArray)
        if call.arguments.len() != 2 {
            return;
        }

        // Skip if the receiver is already `Reflect` (i.e. `Reflect.apply(...)`)
        if let Expression::Identifier(ident) = &member.object {
            if ident.name.as_str() == "Reflect" {
                return;
            }
        }

        ctx.report_warning(
            "prefer-reflect-apply",
            "Use `Reflect.apply()` instead of `.apply()`",
            Span::new(call.span.start, call.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferReflectApply)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_apply_with_null() {
        let diags = lint("foo.apply(null, args);");
        assert_eq!(diags.len(), 1, "foo.apply(null, args) should be flagged");
    }

    #[test]
    fn test_flags_apply_with_this_arg() {
        let diags = lint("foo.apply(thisArg, args);");
        assert_eq!(diags.len(), 1, "foo.apply(thisArg, args) should be flagged");
    }

    #[test]
    fn test_allows_apply_with_one_arg() {
        let diags = lint("foo.apply(thisArg);");
        assert!(
            diags.is_empty(),
            "foo.apply(thisArg) with one arg should not be flagged"
        );
    }

    #[test]
    fn test_allows_call() {
        let diags = lint("foo.call(thisArg, a, b);");
        assert!(diags.is_empty(), "foo.call() should not be flagged");
    }

    #[test]
    fn test_allows_reflect_apply() {
        let diags = lint("Reflect.apply(foo, null, args);");
        assert!(diags.is_empty(), "Reflect.apply() should not be flagged");
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("foo(1, 2);");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }
}
