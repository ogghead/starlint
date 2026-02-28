//! Rule: `no-array-push-push` (unicorn)
//!
//! Flags consecutive `.push()` calls on the same array that could be merged
//! into a single `.push(a, b)` call.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// State tracking the previous `.push()` call site.
#[derive(Debug, Clone)]
struct PushInfo {
    /// Name of the array being pushed to.
    array_name: String,
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
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Only react to ExpressionStatement nodes. During depth-first traversal,
        // child nodes (CallExpression, Identifier, etc.) are visited between
        // sibling ExpressionStatements. We ignore those entirely so that state
        // from a previous push is preserved until the next statement-level node.
        let AstKind::ExpressionStatement(stmt) = kind else {
            return;
        };

        // Try to extract an `arr.push(...)` pattern from this statement.
        let push_target = extract_push_target(&stmt.expression);

        match push_target {
            Some(array_name) => {
                // Check if the previous statement was also a push on the same array.
                let should_report = self
                    .last_push
                    .read()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .as_ref()
                            .filter(|prev| prev.array_name == array_name)
                            .map(|_| true)
                    })
                    .unwrap_or(false);

                if should_report {
                    ctx.report_warning(
                        "no-array-push-push",
                        &format!(
                            "Consecutive `.push()` calls on `{array_name}` can be merged into one"
                        ),
                        Span::new(stmt.span.start, stmt.span.end),
                    );
                }

                // Record this push for the next iteration.
                if let Ok(mut guard) = self.last_push.write() {
                    *guard = Some(PushInfo {
                        array_name: array_name.to_owned(),
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

/// If the expression is `identifier.push(...)`, return the identifier name.
fn extract_push_target<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
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

    Some(id.name.as_str())
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
