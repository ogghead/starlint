//! Rule: `jest/prefer-hooks-on-top`
//!
//! Warn when hooks (`beforeEach`, `afterEach`, `beforeAll`, `afterAll`) appear
//! after test cases in a `describe` block. Hooks should be declared before
//! any test cases for readability.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags hooks that appear after `it`/`test` calls in a describe block.
#[derive(Debug)]
pub struct PreferHooksOnTop;

/// Hook function names that should appear before tests.
const HOOK_NAMES: &[&str] = &["beforeAll", "beforeEach", "afterEach", "afterAll"];

/// Test function names.
const TEST_NAMES: &[&str] = &["it", "test"];

impl NativeRule for PreferHooksOnTop {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-hooks-on-top".to_owned(),
            description: "Warn when hooks are not at the top of the describe block".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be `describe(...)` call
        let is_describe = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "describe"
        );
        if !is_describe {
            return;
        }

        // Get the callback body
        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };
        let Some(callback_expr) = second_arg.as_expression() else {
            return;
        };

        let body = match callback_expr {
            Expression::ArrowFunctionExpression(arrow) => &arrow.body,
            Expression::FunctionExpression(func) => {
                let Some(ref body) = func.body else {
                    return;
                };
                body
            }
            _ => return,
        };

        // Scan statements: once we see a test, any subsequent hook is out of place
        let mut seen_test = false;
        for stmt in &body.statements {
            let Statement::ExpressionStatement(expr_stmt) = stmt else {
                continue;
            };
            let Expression::CallExpression(inner_call) = &expr_stmt.expression else {
                continue;
            };
            let callee_name = match &inner_call.callee {
                Expression::Identifier(id) => id.name.as_str(),
                _ => continue,
            };

            if TEST_NAMES.contains(&callee_name) {
                seen_test = true;
            } else if seen_test && HOOK_NAMES.contains(&callee_name) {
                ctx.report(Diagnostic {
                    rule_name: "jest/prefer-hooks-on-top".to_owned(),
                    message: format!(
                        "`{callee_name}` should be declared before any test cases in the describe block"
                    ),
                    span: Span::new(inner_call.span.start, inner_call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferHooksOnTop)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_hook_after_test() {
        let source = r"
describe('suite', () => {
    test('first', () => {});
    beforeEach(() => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`beforeEach` after `test` should be flagged"
        );
    }

    #[test]
    fn test_allows_hooks_before_tests() {
        let source = r"
describe('suite', () => {
    beforeEach(() => {});
    afterEach(() => {});
    test('first', () => {});
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "hooks before tests should not be flagged");
    }

    #[test]
    fn test_flags_after_all_after_test() {
        let source = r"
describe('suite', () => {
    it('works', () => {});
    afterAll(() => {});
});
";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`afterAll` after `it` should be flagged");
    }
}
