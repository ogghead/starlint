//! Rule: `bad-array-method-on-arguments` (OXC)
//!
//! Detect calling array methods on the `arguments` object. The `arguments`
//! object is not a real array, so methods like `.map()`, `.filter()`, etc.
//! will fail at runtime.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Array methods that don't exist on the `arguments` object.
const ARRAY_METHODS: &[&str] = &[
    "map",
    "filter",
    "reduce",
    "reduceRight",
    "forEach",
    "some",
    "every",
    "find",
    "findIndex",
    "flat",
    "flatMap",
    "includes",
    "indexOf",
    "lastIndexOf",
    "fill",
    "copyWithin",
    "entries",
    "keys",
    "values",
    "from",
];

/// Flags array methods called on `arguments`.
#[derive(Debug)]
pub struct BadArrayMethodOnArguments;

impl NativeRule for BadArrayMethodOnArguments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-array-method-on-arguments".to_owned(),
            description: "Detect array methods called on `arguments`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        // Check if the object is `arguments`
        let is_arguments = matches!(
            &member.object,
            Expression::Identifier(id) if id.name.as_str() == "arguments"
        );

        if !is_arguments {
            return;
        }

        let method = member.property.name.as_str();
        if ARRAY_METHODS.contains(&method) {
            ctx.report_error(
                "bad-array-method-on-arguments",
                &format!(
                    "`arguments.{method}()` will fail — `arguments` is not an array. \
                     Use `Array.from(arguments).{method}()` or rest parameters instead"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadArrayMethodOnArguments)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_arguments_map() {
        let diags = lint("function f() { arguments.map(x => x); }");
        assert_eq!(diags.len(), 1, "arguments.map() should be flagged");
    }

    #[test]
    fn test_flags_arguments_filter() {
        let diags = lint("function f() { arguments.filter(Boolean); }");
        assert_eq!(diags.len(), 1, "arguments.filter() should be flagged");
    }

    #[test]
    fn test_allows_arguments_length() {
        let diags = lint("function f() { return arguments.length; }");
        assert!(diags.is_empty(), "arguments.length should not be flagged");
    }

    #[test]
    fn test_allows_array_map() {
        let diags = lint("var result = arr.map(x => x);");
        assert!(diags.is_empty(), "normal array.map should not be flagged");
    }
}
