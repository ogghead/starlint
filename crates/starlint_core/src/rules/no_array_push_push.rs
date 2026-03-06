//! Rule: `no-array-push-push` (unicorn)
//!
//! Flags consecutive `.push()` calls on the same array that could be merged
//! into a single `.push(a, b)` call.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// State tracking the previous `.push()` call site.
#[derive(Debug, Clone)]
struct PushInfo {
    /// Name of the array being pushed to.
    array_name: String,
    /// Position of the closing `)` of the previous push call's arguments.
    args_close_paren: u32,
    /// Whether the previous push had arguments.
    has_args: bool,
}

/// Flags consecutive `.push()` calls on the same array.
#[derive(Debug)]
pub struct NoArrayPushPush {
    /// Tracks the most recent `.push()` call in the current statement sequence.
    last_push: RwLock<Option<PushInfo>>,
}

impl NoArrayPushPush {
    /// Create a new `NoArrayPushPush` rule.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_push: RwLock::new(None),
        }
    }
}

impl Default for NoArrayPushPush {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for NoArrayPushPush {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-array-push-push".to_owned(),
            description: "Merge consecutive `.push()` calls on the same array".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExpressionStatement])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Only react to ExpressionStatement nodes. During depth-first traversal,
        // child nodes (CallExpression, Identifier, etc.) are visited between
        // sibling ExpressionStatements. We ignore those entirely so that state
        // from a previous push is preserved until the next statement-level node.
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };

        // Try to extract an `arr.push(...)` pattern from this statement.
        let push_info = extract_push_info(&stmt.expression);

        match push_info {
            Some((array_name, args_close_paren, has_args)) => {
                // Check if the previous statement was also a push on the same array.
                // Extract fix data before borrowing ctx mutably.
                let fix_data = self.last_push.read().ok().and_then(|guard| {
                    guard.as_ref().and_then(|prev| {
                        (prev.array_name == array_name)
                            .then_some((prev.args_close_paren, prev.has_args))
                    })
                });

                if let Some((prev_close_paren, prev_has_args)) = fix_data {
                    // Build fix: insert current args into previous push, delete current statement
                    let source = ctx.source_text();
                    let fix = extract_call_args_text(source, &stmt.expression).map(|args_text| {
                        let separator = if prev_has_args { ", " } else { "" };
                        Fix {
                            message: "Merge into previous `.push()` call".to_owned(),
                            edits: vec![
                                // Insert args before the closing paren of the previous push
                                Edit {
                                    span: Span::new(prev_close_paren, prev_close_paren),
                                    replacement: format!("{separator}{args_text}"),
                                },
                                // Delete the current push statement
                                Edit {
                                    span: Span::new(stmt.span.start, stmt.span.end),
                                    replacement: String::new(),
                                },
                            ],
                            is_snippet: false,
                        }
                    });

                    ctx.report(Diagnostic {
                        rule_name: "no-array-push-push".to_owned(),
                        message: format!(
                            "Consecutive `.push()` calls on `{array_name}` can be merged into one"
                        ),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix,
                        labels: vec![],
                    });
                }

                // Record this push for the next iteration.
                if let Ok(mut guard) = self.last_push.write() {
                    *guard = Some(PushInfo {
                        array_name: array_name.to_owned(),
                        args_close_paren,
                        has_args,
                    });
                }
            }
            None => {
                // This ExpressionStatement is not a push call; break the chain.
                if let Ok(mut guard) = self.last_push.write() {
                    *guard = None;
                }
            }
        }
    }
}

/// If the expression is `identifier.push(...)`, return (identifier name, close paren pos, `has_args`).
fn extract_push_info<'a>(expr: &'a Expression<'a>) -> Option<(&'a str, u32, bool)> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };

    if member.property.name.as_str() != "push" {
        return None;
    }

    let Expression::Identifier(id) = &member.object else {
        return None;
    };

    // The closing paren is 1 before the end of the call expression span
    let close_paren = call.span.end.saturating_sub(1);
    let has_args = !call.arguments.is_empty();

    Some((id.name.as_str(), close_paren, has_args))
}

/// Extract the source text of the arguments to a `.push()` call.
#[allow(clippy::as_conversions)]
fn extract_call_args_text<'a>(source: &'a str, expr: &Expression<'_>) -> Option<&'a str> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };

    if call.arguments.is_empty() {
        return Some("");
    }

    let first = call.arguments.first()?;
    let last = call.arguments.last()?;
    let start = first.span().start as usize;
    let end = last.span().end as usize;
    source.get(start..end)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayPushPush::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_consecutive_push() {
        let diags = lint("arr.push(1); arr.push(2);");
        assert!(
            !diags.is_empty(),
            "consecutive push on same array should be flagged"
        );
    }

    #[test]
    fn test_flags_three_consecutive_pushes() {
        let diags = lint("arr.push(1); arr.push(2); arr.push(3);");
        assert!(
            diags.len() >= 2,
            "three consecutive pushes should flag at least two"
        );
    }

    #[test]
    fn test_allows_different_arrays() {
        let diags = lint("arr.push(1); other.push(2);");
        assert!(
            diags.is_empty(),
            "push on different arrays should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_push_with_multiple_args() {
        let diags = lint("arr.push(1, 2);");
        assert!(
            diags.is_empty(),
            "single push with multiple arguments should not be flagged"
        );
    }

    #[test]
    fn test_allows_push_separated_by_other_statement() {
        let diags = lint("arr.push(1); doSomething(); arr.push(2);");
        assert!(
            diags.is_empty(),
            "push calls separated by another statement should not be flagged"
        );
    }

    #[test]
    fn test_allows_single_push() {
        let diags = lint("arr.push(1);");
        assert!(diags.is_empty(), "single push should not be flagged");
    }

    #[test]
    fn test_allows_non_push_method() {
        let diags = lint("arr.pop(); arr.pop();");
        assert!(
            diags.is_empty(),
            "consecutive non-push calls should not be flagged"
        );
    }
}
