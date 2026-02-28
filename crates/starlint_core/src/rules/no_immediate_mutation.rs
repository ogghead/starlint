//! Rule: `no-immediate-mutation`
//!
//! Disallows immediately mutating the result of a method that returns a new
//! array. For example, `arr.filter(x => x > 1).sort()` calls `.sort()` on
//! the new array returned by `.filter()`, which mutates it in place and
//! discards readability. Prefer `toSorted()` or assign to a variable first.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Methods that mutate an array in place.
const MUTATING_METHODS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "sort",
    "reverse",
    "fill",
    "copyWithin",
];

/// Methods that return a new array (the result is safe to use but mutating
/// it immediately is suspicious).
const NEW_ARRAY_METHODS: &[&str] = &[
    "filter",
    "map",
    "slice",
    "concat",
    "flat",
    "flatMap",
    "toSorted",
    "toReversed",
    "toSpliced",
    "with",
];

/// Flags chained calls like `arr.filter(...).sort()` where a mutating method
/// is called immediately on a freshly-created array.
#[derive(Debug)]
pub struct NoImmediateMutation;

impl NativeRule for NoImmediateMutation {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-immediate-mutation".to_owned(),
            description:
                "Disallow immediately mutating the result of a method that returns a new array"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Outer call must be `<expr>.<mutatingMethod>(...)`
        let Expression::StaticMemberExpression(outer_member) = &call.callee else {
            return;
        };

        let mutating_method = outer_member.property.name.as_str();
        if !MUTATING_METHODS.contains(&mutating_method) {
            return;
        }

        // The object of the outer member must be a call expression:
        // `<expr>.<newArrayMethod>(...)`
        let Expression::CallExpression(inner_call) = &outer_member.object else {
            return;
        };

        let Expression::StaticMemberExpression(inner_member) = &inner_call.callee else {
            return;
        };

        let inner_method = inner_member.property.name.as_str();
        if !NEW_ARRAY_METHODS.contains(&inner_method) {
            return;
        }

        ctx.report_warning(
            "no-immediate-mutation",
            &format!(
                "Immediately calling `.{mutating_method}()` on the result of `.{inner_method}()` mutates the new array in place"
            ),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImmediateMutation)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_filter_sort() {
        let diags = lint("[1,2,3].filter(x => x > 1).sort();");
        assert_eq!(diags.len(), 1, ".filter().sort() should be flagged");
    }

    #[test]
    fn test_flags_slice_reverse() {
        let diags = lint("arr.slice().reverse();");
        assert_eq!(diags.len(), 1, ".slice().reverse() should be flagged");
    }

    #[test]
    fn test_flags_map_push() {
        let diags = lint("arr.map(x => x).push(1);");
        assert_eq!(diags.len(), 1, ".map().push() should be flagged");
    }

    #[test]
    fn test_flags_concat_fill() {
        let diags = lint("arr.concat([1]).fill(0);");
        assert_eq!(diags.len(), 1, ".concat().fill() should be flagged");
    }

    #[test]
    fn test_allows_sort_alone() {
        let diags = lint("arr.sort();");
        assert!(diags.is_empty(), "standalone .sort() should not be flagged");
    }

    #[test]
    fn test_allows_push_alone() {
        let diags = lint("arr.push(1);");
        assert!(diags.is_empty(), "standalone .push() should not be flagged");
    }

    #[test]
    fn test_allows_filter_alone() {
        let diags = lint("arr.filter(x => x > 1);");
        assert!(
            diags.is_empty(),
            "standalone .filter() should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_mutating_chain() {
        let diags = lint("arr.filter(x => x > 1).map(x => x * 2);");
        assert!(
            diags.is_empty(),
            "chaining non-mutating methods should not be flagged"
        );
    }
}
