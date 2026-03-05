//! Rule: `no-length-as-slice-end`
//!
//! Flag `.slice(start, X.length)` calls where the second argument is a
//! `.length` member access. When `.length` is used as the end argument,
//! it is equivalent to omitting the second argument entirely since
//! `.slice()` defaults to slicing to the end.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.slice(start, X.length)` patterns where `.length` is redundant.
#[derive(Debug)]
pub struct NoLengthAsSliceEnd;

impl NativeRule for NoLengthAsSliceEnd {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-length-as-slice-end".to_owned(),
            description: "Disallow using `.length` as the end argument in `.slice()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `.slice(...)` member call
        let Expression::StaticMemberExpression(slice_member) = &call.callee else {
            return;
        };

        if slice_member.property.name.as_str() != "slice" {
            return;
        }

        // Must have exactly 2 arguments
        if call.arguments.len() != 2 {
            return;
        }

        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };

        // Check if the second argument is a `.length` member access
        if !is_length_member_access(second_arg) {
            return;
        }

        // Extract the receiver of `.slice()` and the object of `.length`
        // to check if they refer to the same entity
        let slice_receiver_text = extract_source_text(&slice_member.object, ctx.source_text());
        let length_object_text = extract_length_object_source(second_arg, ctx.source_text());

        if let (Some(receiver), Some(length_obj)) = (slice_receiver_text, length_object_text) {
            if receiver == length_obj {
                let call_span = Span::new(call.span.start, call.span.end);
                // Remove from end of first argument to end of second argument
                // This removes ", X.length" from ".slice(start, X.length)"
                let first_arg_end = call.arguments.first().map_or(0, |a| a.span().end);
                let second_arg_end = second_arg.span().end;
                let remove_span = Span::new(first_arg_end, second_arg_end);
                ctx.report(Diagnostic {
                    rule_name: "no-length-as-slice-end".to_owned(),
                    message: "Unnecessary `.length` as `.slice()` end — `.slice()` already defaults to the full length".to_owned(),
                    span: call_span,
                    severity: Severity::Warning,
                    help: Some("Remove the `.length` end argument".to_owned()),
                    fix: Some(Fix {
                        message: "Remove `.length` end argument".to_owned(),
                        edits: vec![Edit {
                            span: remove_span,
                            replacement: String::new(),
                        }],
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Check if an argument is a `.length` static member expression.
fn is_length_member_access(arg: &Argument<'_>) -> bool {
    matches!(
        arg,
        Argument::StaticMemberExpression(member) if member.property.name.as_str() == "length"
    )
}

/// Extract source text for an expression using span offsets.
fn extract_source_text<'a>(expr: &Expression<'_>, source: &'a str) -> Option<&'a str> {
    let span = expr.span();
    let start = usize::try_from(span.start).unwrap_or(0);
    let end = usize::try_from(span.end).unwrap_or(0);
    source.get(start..end)
}

/// Extract the source text of the object in a `.length` member access argument.
fn extract_length_object_source<'a>(arg: &Argument<'_>, source: &'a str) -> Option<&'a str> {
    if let Argument::StaticMemberExpression(member) = arg {
        extract_source_text(&member.object, source)
    } else {
        None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLengthAsSliceEnd)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_slice_with_same_length() {
        let diags = lint("s.slice(0, s.length);");
        assert_eq!(
            diags.len(),
            1,
            "s.slice(0, s.length) should be flagged (same receiver)"
        );
    }

    #[test]
    fn test_flags_array_slice_with_same_length() {
        let diags = lint("arr.slice(1, arr.length);");
        assert_eq!(
            diags.len(),
            1,
            "arr.slice(1, arr.length) should be flagged (same receiver)"
        );
    }

    #[test]
    fn test_allows_slice_no_end() {
        let diags = lint("s.slice(0);");
        assert!(
            diags.is_empty(),
            "s.slice(0) should not be flagged (no end argument)"
        );
    }

    #[test]
    fn test_allows_slice_with_different_object() {
        let diags = lint("a.slice(1, b.length);");
        assert!(
            diags.is_empty(),
            "a.slice(1, b.length) should not be flagged (different objects)"
        );
    }

    #[test]
    fn test_allows_slice_with_numeric_end() {
        let diags = lint("s.slice(0, 5);");
        assert!(
            diags.is_empty(),
            "s.slice(0, 5) should not be flagged (numeric end)"
        );
    }

    #[test]
    fn test_allows_non_slice_call() {
        let diags = lint("s.substring(0, s.length);");
        assert!(
            diags.is_empty(),
            "substring calls should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_slice_with_three_args() {
        // .slice() only takes two args, but if somehow called with more,
        // we should not flag it
        let diags = lint("s.slice(0, s.length, extra);");
        assert!(
            diags.is_empty(),
            "slice with three args should not be flagged"
        );
    }
}
