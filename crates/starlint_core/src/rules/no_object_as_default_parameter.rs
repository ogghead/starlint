//! Rule: `no-object-as-default-parameter`
//!
//! Disallow using object literals as default parameter values. A mutable object
//! literal in a default parameter creates a new object on every call, which can
//! be confusing and wasteful. Prefer destructuring defaults instead:
//! `function foo({ a = 1 } = {})` rather than `function foo(x = { a: 1 })`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags object literals used as default parameter values.
#[derive(Debug)]
pub struct NoObjectAsDefaultParameter;

impl NativeRule for NoObjectAsDefaultParameter {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-object-as-default-parameter".to_owned(),
            description: "Disallow object literals as default parameter values".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrowFunctionExpression, AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let params = match kind {
            AstKind::Function(f) => &f.params,
            AstKind::ArrowFunctionExpression(arrow) => &arrow.params,
            _ => return,
        };

        for param in &params.items {
            let Some(init) = &param.initializer else {
                continue;
            };

            // Only flag non-empty object expressions as defaults.
            // `{ a = 1 } = {}` (empty object default for destructured param) is fine.
            let Expression::ObjectExpression(obj) = init.as_ref() else {
                continue;
            };

            if obj.properties.is_empty() {
                continue;
            }

            ctx.report(Diagnostic {
                rule_name: "no-object-as-default-parameter".to_owned(),
                message: "Do not use an object literal as a default parameter — prefer destructuring defaults".to_owned(),
                span: Span::new(param.span.start, param.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoObjectAsDefaultParameter)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_literal_default() {
        let diags = lint("function foo(x = { a: 1 }) {}");
        assert_eq!(
            diags.len(),
            1,
            "object literal as default param should be flagged"
        );
    }

    #[test]
    fn test_flags_object_literal_default_multiple_props() {
        let diags = lint("function foo(x = { a: 1, b: 2 }) {}");
        assert_eq!(
            diags.len(),
            1,
            "object literal with multiple props as default should be flagged"
        );
    }

    #[test]
    fn test_allows_array_default() {
        let diags = lint("function foo(x = []) {}");
        assert!(diags.is_empty(), "array default should not be flagged");
    }

    #[test]
    fn test_allows_string_default() {
        let diags = lint("function foo(x = 'default') {}");
        assert!(diags.is_empty(), "string default should not be flagged");
    }

    #[test]
    fn test_allows_empty_object_for_destructured_param() {
        let diags = lint("function foo({ a = 1 } = {}) {}");
        assert!(
            diags.is_empty(),
            "empty object as default for destructured param should not be flagged"
        );
    }

    #[test]
    fn test_allows_numeric_default() {
        let diags = lint("function foo(x = 42) {}");
        assert!(diags.is_empty(), "numeric default should not be flagged");
    }

    #[test]
    fn test_allows_null_default() {
        let diags = lint("function foo(x = null) {}");
        assert!(diags.is_empty(), "null default should not be flagged");
    }

    #[test]
    fn test_flags_arrow_function_object_default() {
        let diags = lint("const foo = (x = { a: 1 }) => {};");
        assert_eq!(
            diags.len(),
            1,
            "object literal default in arrow function should be flagged"
        );
    }
}
