//! Rule: `no-unnecessary-array-flat-depth`
//!
//! Flag `.flat(1)` calls since `1` is the default depth for
//! `Array.prototype.flat()`. Calling `.flat()` without an argument is
//! equivalent and more concise.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.flat(1)` calls where the depth argument is the default value.
#[derive(Debug)]
pub struct NoUnnecessaryArrayFlatDepth;

impl NativeRule for NoUnnecessaryArrayFlatDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-array-flat-depth".to_owned(),
            description: "Disallow passing the default depth `1` to `.flat()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be a `.flat()` call
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "flat" {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // The argument must be the numeric literal `1`
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        if is_numeric_one(first_arg) {
            ctx.report_warning(
                "no-unnecessary-array-flat-depth",
                "Unnecessary depth argument — `.flat()` defaults to depth `1`",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if an argument is the numeric literal `1`.
fn is_numeric_one(arg: &Argument<'_>) -> bool {
    matches!(arg, Argument::NumericLiteral(n) if (n.value - 1.0).abs() < f64::EPSILON)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnnecessaryArrayFlatDepth)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_flat_with_one() {
        let diags = lint("arr.flat(1);");
        assert_eq!(diags.len(), 1, "arr.flat(1) should be flagged");
    }

    #[test]
    fn test_allows_flat_without_argument() {
        let diags = lint("arr.flat();");
        assert!(diags.is_empty(), "arr.flat() should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_two() {
        let diags = lint("arr.flat(2);");
        assert!(diags.is_empty(), "arr.flat(2) should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_infinity() {
        let diags = lint("arr.flat(Infinity);");
        assert!(diags.is_empty(), "arr.flat(Infinity) should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_zero() {
        let diags = lint("arr.flat(0);");
        assert!(diags.is_empty(), "arr.flat(0) should not be flagged");
    }

    #[test]
    fn test_allows_flat_with_variable() {
        let diags = lint("arr.flat(depth);");
        assert!(diags.is_empty(), "arr.flat(depth) should not be flagged");
    }
}
