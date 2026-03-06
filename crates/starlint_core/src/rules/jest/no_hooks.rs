//! Rule: `jest/no-hooks`
//!
//! Warn when lifecycle hooks (`beforeEach`, `afterEach`, `beforeAll`, `afterAll`) are used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-hooks";

/// Hook names that this rule flags.
const HOOK_NAMES: &[&str] = &["beforeEach", "afterEach", "beforeAll", "afterAll"];

/// Flags usage of Jest lifecycle hooks.
#[derive(Debug)]
pub struct NoHooks;

impl NativeRule for NoHooks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow usage of Jest lifecycle hooks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if HOOK_NAMES.contains(&callee_name) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Unexpected use of `{callee_name}` hook — prefer explicit setup in each test"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoHooks)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_before_each() {
        let diags = lint("beforeEach(() => { setup(); });");
        assert_eq!(diags.len(), 1, "`beforeEach` should be flagged");
    }

    #[test]
    fn test_flags_after_all() {
        let diags = lint("afterAll(() => { cleanup(); });");
        assert_eq!(diags.len(), 1, "`afterAll` should be flagged");
    }

    #[test]
    fn test_allows_regular_calls() {
        let diags = lint("test('works', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "regular test calls should not be flagged");
    }
}
