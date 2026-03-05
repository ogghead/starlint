//! Rule: `no-array-method-this-argument`
//!
//! Disallows using the `thisArg` parameter on array iteration methods.
//! Methods like `map`, `filter`, `some`, `every`, `find`, etc. accept
//! an optional second argument that sets `this` inside the callback.
//! Modern JavaScript should use arrow functions (which capture `this`
//! lexically) instead of relying on `thisArg`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Array methods that accept a `thisArg` as their second parameter.
const METHODS_WITH_THIS_ARG: &[&str] = &[
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
];

/// Flags array method calls that pass a `thisArg` second argument.
#[derive(Debug)]
pub struct NoArrayMethodThisArgument;

impl NativeRule for NoArrayMethodThisArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-method-this-argument".to_owned(),
            description: "Disallow using the thisArg parameter on array iteration methods"
                .to_owned(),
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

        let method_name = member.property.name.as_str();
        if !METHODS_WITH_THIS_ARG.contains(&method_name) {
            return;
        }

        // These methods accept (callback, thisArg) — flag when more than 1 argument
        if call.arguments.len() <= 1 {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-array-method-this-argument".to_owned(),
            message: format!(
                "Do not use the `thisArg` parameter of `.{method_name}()` — use an arrow function instead"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayMethodThisArgument)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_map_with_this_arg() {
        let diags = lint("arr.map(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.map(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_filter_with_this_arg() {
        let diags = lint("arr.filter(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.filter(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_every_with_this_arg() {
        let diags = lint("arr.every(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.every(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_find_with_this_arg() {
        let diags = lint("arr.find(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.find(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_flags_some_with_this_arg() {
        let diags = lint("arr.some(fn, thisArg);");
        assert_eq!(diags.len(), 1, "arr.some(fn, thisArg) should be flagged");
    }

    #[test]
    fn test_allows_map_without_this_arg() {
        let diags = lint("arr.map(fn);");
        assert!(diags.is_empty(), "arr.map(fn) should not be flagged");
    }

    #[test]
    fn test_allows_reduce_with_initial_value() {
        let diags = lint("arr.reduce(fn, init);");
        assert!(
            diags.is_empty(),
            "arr.reduce(fn, init) should not be flagged (second arg is initial value)"
        );
    }

    #[test]
    fn test_allows_arrow_function_callback() {
        let diags = lint("arr.map(x => x * 2);");
        assert!(
            diags.is_empty(),
            "arrow function callback without thisArg should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("arr.indexOf(value, fromIndex);");
        assert!(
            diags.is_empty(),
            "indexOf with two args should not be flagged"
        );
    }
}
