//! Rule: `no-array-callback-reference`
//!
//! Disallows passing function references directly to array iteration methods.
//! When a function reference is passed (e.g. `arr.map(parseInt)`), the
//! iteration method passes extra arguments (`index`, `array`) that the
//! function may not expect. For instance, `parseInt` interprets the second
//! argument as a radix, causing `["1","2","3"].map(parseInt)` to produce
//! `[1, NaN, NaN]`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Array methods whose callbacks receive extra positional arguments.
const ITERATION_METHODS: &[&str] = &[
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flatMap",
    "forEach",
    "map",
    "some",
    "sort",
    "reduce",
    "reduceRight",
];

/// Flags function references passed directly to array iteration methods.
#[derive(Debug)]
pub struct NoArrayCallbackReference;

impl NativeRule for NoArrayCallbackReference {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-callback-reference".to_owned(),
            description: "Disallow passing function references directly to array iteration methods"
                .to_owned(),
            category: Category::Suggestion,
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

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();
        if !ITERATION_METHODS.contains(&method_name) {
            return;
        }

        // Must have at least one argument
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Flag only if the first argument is a bare identifier reference
        // (not an arrow function, function expression, or other expression)
        if let Argument::Identifier(id) = first_arg {
            let fn_name = id.name.as_str();
            ctx.report_warning(
                "no-array-callback-reference",
                &format!(
                    "Do not pass `{fn_name}` directly to `.{method_name}()` — it may receive unexpected arguments"
                ),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayCallbackReference)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_map_parse_int() {
        let diags = lint("arr.map(parseInt);");
        assert_eq!(diags.len(), 1, "arr.map(parseInt) should be flagged");
    }

    #[test]
    fn test_flags_filter_boolean() {
        let diags = lint("arr.filter(Boolean);");
        assert_eq!(diags.len(), 1, "arr.filter(Boolean) should be flagged");
    }

    #[test]
    fn test_flags_some_with_identifier() {
        let diags = lint("arr.some(isValid);");
        assert_eq!(diags.len(), 1, "arr.some(isValid) should be flagged");
    }

    #[test]
    fn test_flags_reduce_with_identifier() {
        let diags = lint("arr.reduce(merge);");
        assert_eq!(diags.len(), 1, "arr.reduce(merge) should be flagged");
    }

    #[test]
    fn test_allows_arrow_function() {
        let diags = lint("arr.map(x => parseInt(x));");
        assert!(
            diags.is_empty(),
            "arrow function callback should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_expression() {
        let diags = lint("arr.map(function(x) { return x; });");
        assert!(
            diags.is_empty(),
            "function expression callback should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("arr.indexOf(parseInt);");
        assert!(
            diags.is_empty(),
            "non-iteration method should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_arguments() {
        let diags = lint("arr.sort();");
        assert!(
            diags.is_empty(),
            "iteration method without arguments should not be flagged"
        );
    }
}
