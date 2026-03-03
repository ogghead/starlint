//! Rule: `jest/no-conditional-in-test`
//!
//! Warn when if/switch/ternary is used inside test callbacks.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-conditional-in-test";

/// Flags conditional statements inside test callbacks.
///
/// Simplified: detects `if`/`switch`/ternary at the source level within test
/// file context. The rule matches `AstKind::IfStatement`, `AstKind::SwitchStatement`,
/// and `AstKind::ConditionalExpression` and checks if they appear inside a
/// `test`/`it` callback by scanning the preceding source.
#[derive(Debug)]
pub struct NoConditionalInTest;

impl NativeRule for NoConditionalInTest {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow conditional logic in tests".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ConditionalExpression,
            AstType::IfStatement,
            AstType::SwitchStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let (stmt_type, span_start, span_end) = match kind {
            AstKind::IfStatement(stmt) => ("if statement", stmt.span.start, stmt.span.end),
            AstKind::SwitchStatement(stmt) => ("switch statement", stmt.span.start, stmt.span.end),
            AstKind::ConditionalExpression(expr) => {
                ("ternary expression", expr.span.start, expr.span.end)
            }
            _ => return,
        };

        // Check if this conditional is inside a test callback by scanning the source
        // before it for `test(` or `it(` patterns
        let source = ctx.source_text();
        let pos = usize::try_from(span_start).unwrap_or(0);
        let before = source.get(..pos).unwrap_or("");

        if is_inside_test_callback(before) {
            ctx.report_warning(
                RULE_NAME,
                &format!("Unexpected {stmt_type} inside a test — tests should not contain conditional logic"),
                Span::new(span_start, span_end),
            );
        }
    }
}

/// Check if a position is inside a test/it callback by counting open/close braces
/// after the last `test(` or `it(` call.
fn is_inside_test_callback(before: &str) -> bool {
    // Find the last occurrence of `test(` or `it(`
    let last_test = before.rfind("test(");
    let last_it = before.rfind("it(");

    let call_pos = match (last_test, last_it) {
        (Some(t), Some(i)) => Some(t.max(i)),
        (Some(t), None) => Some(t),
        (None, Some(i)) => Some(i),
        (None, None) => None,
    };

    let Some(pos) = call_pos else {
        return false;
    };

    // Count braces from the call position to see if we're still inside the callback
    let after_call = before.get(pos..).unwrap_or("");
    let mut brace_depth: i32 = 0;
    for ch in after_call.chars() {
        if ch == '{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
        }
    }

    // If braces are still open, we're inside the callback
    brace_depth > 0
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConditionalInTest)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_if_in_test() {
        let source = "test('cond', () => { if (true) { console.log('x'); } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "if inside test should be flagged");
    }

    #[test]
    fn test_flags_ternary_in_test() {
        let source = "it('cond', () => { const x = true ? 1 : 2; });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "ternary inside test should be flagged");
    }

    #[test]
    fn test_allows_if_outside_test() {
        let source = "if (process.env.CI) { console.log('ci'); }";
        let diags = lint(source);
        assert!(diags.is_empty(), "if outside test should not be flagged");
    }
}
