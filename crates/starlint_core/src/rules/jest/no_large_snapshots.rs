//! Rule: `jest/no-large-snapshots`
//!
//! Warn when inline snapshot strings are too long (> 50 lines).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-large-snapshots";

/// Maximum number of lines allowed in an inline snapshot.
const MAX_LINES: usize = 50;

/// Flags inline snapshot arguments that exceed the line threshold.
#[derive(Debug)]
pub struct NoLargeSnapshots;

impl NativeRule for NoLargeSnapshots {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow large inline snapshots".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `.toMatchInlineSnapshot(...)` pattern
        let is_inline_snapshot = match &call.callee {
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "toMatchInlineSnapshot"
            }
            _ => false,
        };

        if !is_inline_snapshot {
            return;
        }

        // Check the first argument — should be a string literal or template literal
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let line_count = match first_arg.as_expression() {
            Some(Expression::StringLiteral(s)) => count_lines(s.value.as_str()),
            Some(Expression::TemplateLiteral(t)) => {
                // Count lines across all quasis (template string parts)
                let mut lines: usize = 0;
                for quasi in &t.quasis {
                    let raw = quasi.value.raw.as_str();
                    lines = lines.saturating_add(count_lines(raw));
                }
                lines
            }
            _ => return,
        };

        if line_count > MAX_LINES {
            ctx.report_warning(
                RULE_NAME,
                &format!(
                    "Inline snapshot is too large ({line_count} lines, max {MAX_LINES}) — use an external snapshot file instead"
                ),
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Count the number of lines in a string.
fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        return 1;
    }
    s.lines().count().max(1)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLargeSnapshots)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_large_inline_snapshot() {
        // Generate a string with 60 lines
        let big_snapshot: String = (0..60)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\\n");
        let source = format!("expect(result).toMatchInlineSnapshot(\"{big_snapshot}\");");
        let diags = lint(&source);
        assert_eq!(diags.len(), 1, "large inline snapshot should be flagged");
    }

    #[test]
    fn test_allows_small_inline_snapshot() {
        let source = r#"expect(result).toMatchInlineSnapshot("small value");"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "small inline snapshot should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_snapshot_call() {
        let diags = lint("expect(result).toBe(true);");
        assert!(
            diags.is_empty(),
            "non-snapshot matcher should not be flagged"
        );
    }
}
