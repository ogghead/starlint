//! Rule: `jest/no-test-return-statement`
//!
//! Warn when a test callback has a return statement.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-test-return-statement";

/// Flags return statements inside test callbacks.
#[derive(Debug)]
pub struct NoTestReturnStatement;

impl NativeRule for NoTestReturnStatement {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow return statements in test callbacks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ReturnStatement(ret) = kind else {
            return;
        };

        // Check if this return is inside a test/it callback
        let source = ctx.source_text();
        let pos = usize::try_from(ret.span.start).unwrap_or(0);
        let before = source.get(..pos).unwrap_or("");

        if is_inside_test_callback(before) {
            ctx.report_warning(
                RULE_NAME,
                "Unexpected return statement in test — tests should not return values",
                Span::new(ret.span.start, ret.span.end),
            );
        }
    }
}

/// Check if a position is inside a test/it callback by counting brace depth.
fn is_inside_test_callback(before: &str) -> bool {
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

    let after_call = before.get(pos..).unwrap_or("");
    let mut brace_depth: i32 = 0;
    for ch in after_call.chars() {
        if ch == '{' {
            brace_depth = brace_depth.saturating_add(1);
        } else if ch == '}' {
            brace_depth = brace_depth.saturating_sub(1);
        }
    }

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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTestReturnStatement)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_in_test() {
        let source = "test('returns', () => { return 42; });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "return in test callback should be flagged");
    }

    #[test]
    fn test_allows_test_without_return() {
        let source = "test('ok', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "test without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_outside_test() {
        let source = "function helper() { return 1; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "return outside test should not be flagged"
        );
    }
}
