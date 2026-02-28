//! Rule: `jest/no-standalone-expect`
//!
//! Error when `expect()` is used outside of `it`/`test` callbacks.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-standalone-expect";

/// Flags `expect()` calls that appear outside of test/it callbacks.
#[derive(Debug)]
pub struct NoStandaloneExpect;

impl NativeRule for NoStandaloneExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `expect()` outside of `it`/`test` blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check callee is `expect`
        let is_expect = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "expect"
        );

        if !is_expect {
            return;
        }

        // Check if this expect is inside a test/it callback
        let source = ctx.source_text();
        let pos = usize::try_from(call.span.start).unwrap_or(0);
        let before = source.get(..pos).unwrap_or("");

        if !is_inside_test_callback(before) {
            ctx.report_error(
                RULE_NAME,
                "`expect()` must be called inside an `it()` or `test()` block",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if a position is inside a test/it callback by finding the last
/// `test(`/`it(` call and counting brace depth.
fn is_inside_test_callback(before: &str) -> bool {
    let last_test = before.rfind("test(");
    let last_it = before.rfind("it(");

    // Also consider beforeEach/afterEach/beforeAll/afterAll as valid containers
    let last_before_each = before.rfind("beforeEach(");
    let last_after_each = before.rfind("afterEach(");
    let last_before_all = before.rfind("beforeAll(");
    let last_after_all = before.rfind("afterAll(");

    let call_pos = [
        last_test,
        last_it,
        last_before_each,
        last_after_each,
        last_before_all,
        last_after_all,
    ]
    .into_iter()
    .flatten()
    .max();

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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoStandaloneExpect)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_standalone_expect() {
        let diags = lint("expect(true).toBe(true);");
        assert_eq!(diags.len(), 1, "standalone expect should be flagged");
    }

    #[test]
    fn test_allows_expect_in_test() {
        let diags = lint("test('ok', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "expect inside test should not be flagged");
    }

    #[test]
    fn test_allows_expect_in_it() {
        let diags = lint("it('ok', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "expect inside it should not be flagged");
    }
}
