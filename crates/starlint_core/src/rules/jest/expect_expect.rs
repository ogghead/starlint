//! Rule: `jest/expect-expect`
//!
//! Warn when a test has no `expect()` call inside its callback body.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/expect-expect";

/// Flags test blocks (`it`/`test`) that contain no `expect()` calls.
#[derive(Debug)]
pub struct ExpectExpect;

impl NativeRule for ExpectExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require at least one `expect()` call in each test".to_owned(),
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

        // Check callee is `it` or `test`
        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if callee_name != "it" && callee_name != "test" {
            return;
        }

        // Get the callback (second argument)
        let Some(callback) = call.arguments.get(1) else {
            return;
        };

        // Extract the body span to search for `expect(` in source
        let (body_start, body_end) = match callback {
            Argument::ArrowFunctionExpression(arrow) => (arrow.span.start, arrow.span.end),
            Argument::FunctionExpression(func) => (func.span.start, func.span.end),
            _ => return,
        };

        let source = ctx.source_text();
        let start = usize::try_from(body_start).unwrap_or(0);
        let end = usize::try_from(body_end).unwrap_or(0);
        let body_source = source.get(start..end).unwrap_or("");

        if !body_source.contains("expect(") {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Test `{callee_name}()` has no `expect()` call — tests should assert something"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExpectExpect)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_test_without_expect() {
        let diags = lint("test('does nothing', () => { const x = 1; });");
        assert_eq!(diags.len(), 1, "test without expect should be flagged");
    }

    #[test]
    fn test_allows_test_with_expect() {
        let diags = lint("test('works', () => { expect(1).toBe(1); });");
        assert!(diags.is_empty(), "test with expect should not be flagged");
    }

    #[test]
    fn test_flags_it_without_expect() {
        let diags = lint("it('does nothing', () => { console.log('hi'); });");
        assert_eq!(diags.len(), 1, "it() without expect should be flagged");
    }
}
