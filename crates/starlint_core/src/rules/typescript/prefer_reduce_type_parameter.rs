//! Rule: `typescript/prefer-reduce-type-parameter`
//!
//! Prefer specifying the generic type argument on `Array.prototype.reduce()`
//! calls instead of using `as` type assertions on the initial value. Writing
//! `.reduce(fn, init as T)` loses type safety; `.reduce<T>(fn, init)` is
//! clearer and preserves the type contract.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.reduce(fn, init as T)` patterns where the initial value uses a
/// type assertion instead of a generic type parameter.
#[derive(Debug)]
pub struct PreferReduceTypeParameter;

impl NativeRule for PreferReduceTypeParameter {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-reduce-type-parameter".to_owned(),
            description:
                "Prefer using a generic type parameter for `reduce` instead of `as` assertions"
                    .to_owned(),
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

        if member.property.name.as_str() != "reduce" {
            return;
        }

        // Check if any argument is a TSAsExpression (i.e. `value as Type`)
        let has_as_assertion = call
            .arguments
            .iter()
            .any(|arg| matches!(arg, Argument::TSAsExpression(_)));

        if has_as_assertion {
            ctx.report(Diagnostic {
                rule_name: "typescript/prefer-reduce-type-parameter".to_owned(),
                message: "Use a generic type parameter on `.reduce<T>()` instead of asserting the initial value with `as`".to_owned(),
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferReduceTypeParameter)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reduce_with_as_assertion_on_init() {
        let diags =
            lint("const result = arr.reduce((acc, item) => acc, {} as Record<string, number>);");
        assert_eq!(
            diags.len(),
            1,
            "`.reduce()` with `as` on initial value should be flagged"
        );
    }

    #[test]
    fn test_flags_reduce_with_as_assertion_on_any_arg() {
        let diags = lint("const result = arr.reduce((acc, item) => acc, [] as string[]);");
        assert_eq!(
            diags.len(),
            1,
            "`.reduce()` with `as` assertion should be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_with_generic_type() {
        let diags =
            lint("const result = arr.reduce<Record<string, number>>((acc, item) => acc, {});");
        assert!(
            diags.is_empty(),
            "`.reduce()` with generic type parameter should not be flagged"
        );
    }

    #[test]
    fn test_allows_reduce_without_assertion() {
        let diags = lint("const result = arr.reduce((acc, item) => acc + item, 0);");
        assert!(
            diags.is_empty(),
            "`.reduce()` without type assertion should not be flagged"
        );
    }

    #[test]
    fn test_ignores_non_reduce_method() {
        let diags = lint("const result = arr.map((item) => item as string);");
        assert!(
            diags.is_empty(),
            "non-reduce method calls should not be flagged"
        );
    }
}
