//! Rule: `no-unnecessary-array-splice-count`
//!
//! Flag `.splice(index, arr.length)` where the count argument is the array's
//! `.length`. Since `splice(index)` removes all remaining elements from the
//! given index, passing `.length` as the delete count is redundant.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.splice(index, obj.length)` where the count argument is redundant.
#[derive(Debug)]
pub struct NoUnnecessaryArraySpliceCount;

impl NativeRule for NoUnnecessaryArraySpliceCount {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unnecessary-array-splice-count".to_owned(),
            description: "Disallow redundant `.length` as second argument to `.splice()`"
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

        // Must be a `.splice()` call
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "splice" {
            return;
        }

        // Must have exactly two arguments (index, count)
        // If there are more arguments (replacement elements), the `.length`
        // count is meaningful because it controls how many elements are removed
        // before inserting replacements, so we only flag the two-argument form.
        if call.arguments.len() != 2 {
            return;
        }

        // Second argument must be a `.length` member expression
        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };

        let Some(second_expr) = second_arg.as_expression() else {
            return;
        };

        let Expression::StaticMemberExpression(length_member) = second_expr else {
            return;
        };

        if length_member.property.name.as_str() != "length" {
            return;
        }

        // Compare the source text of the splice receiver and the .length owner
        let receiver_span = member.object.span();
        let length_owner_span = length_member.object.span();

        let receiver_start = usize::try_from(receiver_span.start).unwrap_or(0);
        let receiver_end = usize::try_from(receiver_span.end).unwrap_or(0);
        let owner_start = usize::try_from(length_owner_span.start).unwrap_or(0);
        let owner_end = usize::try_from(length_owner_span.end).unwrap_or(0);

        let source = ctx.source_text();
        let receiver_text = source.get(receiver_start..receiver_end);
        let owner_text = source.get(owner_start..owner_end);

        if let (Some(receiver), Some(owner)) = (receiver_text, owner_text) {
            if !receiver.is_empty() && receiver == owner {
                ctx.report_warning(
                    "no-unnecessary-array-splice-count",
                    &format!(
                        "Unnecessary `.length` argument — `{receiver}.splice(index)` already removes all remaining elements"
                    ),
                    Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnnecessaryArraySpliceCount)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_splice_with_length() {
        let diags = lint("arr.splice(0, arr.length);");
        assert_eq!(
            diags.len(),
            1,
            "arr.splice(0, arr.length) should be flagged"
        );
    }

    #[test]
    fn test_allows_splice_without_count() {
        let diags = lint("arr.splice(0);");
        assert!(diags.is_empty(), "arr.splice(0) should not be flagged");
    }

    #[test]
    fn test_allows_splice_with_numeric_count() {
        let diags = lint("arr.splice(0, 3);");
        assert!(diags.is_empty(), "arr.splice(0, 3) should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("arr.splice(0, other.length);");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }

    #[test]
    fn test_allows_splice_with_replacements() {
        let diags = lint("arr.splice(0, arr.length, 'a', 'b');");
        assert!(
            diags.is_empty(),
            "splice with replacement elements should not be flagged"
        );
    }
}
