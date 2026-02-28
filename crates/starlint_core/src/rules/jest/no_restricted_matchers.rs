//! Rule: `jest/no-restricted-matchers`
//!
//! Warn when restricted matchers are used (e.g., `.toBeTruthy()`, `.toBeFalsy()`).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-restricted-matchers";

/// Default restricted matchers. These are commonly flagged because they produce
/// less informative test failures compared to explicit matchers.
const RESTRICTED_MATCHERS: &[&str] = &[
    "toBeTruthy",
    "toBeFalsy",
    "resolves",
    "rejects",
    "toMatchSnapshot",
];

/// Flags usage of restricted Jest matchers in expect chains.
#[derive(Debug)]
pub struct NoRestrictedMatchers;

impl NativeRule for NoRestrictedMatchers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow restricted Jest matchers".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Match `expect(...).matcher(...)` or `expect(...).not.matcher(...)` or
        // `expect(...).resolves/rejects`
        let matcher_name = match &call.callee {
            Expression::StaticMemberExpression(member) => member.property.name.as_str(),
            _ => return,
        };

        if !RESTRICTED_MATCHERS.contains(&matcher_name) {
            return;
        }

        // Verify this is part of an expect chain by walking up the member expression
        let is_expect_chain = is_in_expect_chain(&call.callee);

        if is_expect_chain {
            ctx.report_warning(
                RULE_NAME,
                &format!("`.{matcher_name}` matcher is restricted — use a more specific matcher"),
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check whether a callee expression is part of an `expect(...)` chain.
/// Walks through member expression objects looking for `expect(...)`.
fn is_in_expect_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StaticMemberExpression(member) => is_expect_call_or_chain(&member.object),
        _ => false,
    }
}

/// Recursively check if an expression is `expect(...)` or a chain from it.
fn is_expect_call_or_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => {
            // Direct `expect(...)` call
            matches!(
                &call.callee,
                Expression::Identifier(id) if id.name.as_str() == "expect"
            )
        }
        Expression::StaticMemberExpression(member) => {
            // `expect(...).not` or `expect(...).resolves` etc.
            is_expect_call_or_chain(&member.object)
        }
        _ => false,
    }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedMatchers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_be_truthy() {
        let diags = lint("expect(value).toBeTruthy();");
        assert_eq!(
            diags.len(),
            1,
            "`.toBeTruthy()` should be flagged as restricted"
        );
    }

    #[test]
    fn test_flags_to_be_falsy() {
        let diags = lint("expect(value).toBeFalsy();");
        assert_eq!(
            diags.len(),
            1,
            "`.toBeFalsy()` should be flagged as restricted"
        );
    }

    #[test]
    fn test_allows_to_be() {
        let diags = lint("expect(value).toBe(true);");
        assert!(
            diags.is_empty(),
            "`.toBe()` should not be flagged as it is not restricted"
        );
    }
}
