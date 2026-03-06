//! Rule: `jest/prefer-hooks-in-order`
//!
//! Warn when hooks are not in the standard order: `beforeAll`, `beforeEach`,
//! `afterEach`, `afterAll`. Consistent ordering improves readability and
//! makes the lifecycle flow explicit.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags hooks that are not in the standard lifecycle order.
#[derive(Debug)]
pub struct PreferHooksInOrder;

/// Expected hook order (lower index = should come first).
const HOOK_ORDER: &[&str] = &["beforeAll", "beforeEach", "afterEach", "afterAll"];

impl NativeRule for PreferHooksInOrder {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-hooks-in-order".to_owned(),
            description: "Warn when hooks are not in the standard lifecycle order".to_owned(),
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

        // Collect hooks with their order index and span
        let mut last_order: Option<usize> = None;
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

            let Some(order) = HOOK_ORDER.iter().position(|&h| h == callee_name) else {
                continue;
            };

            if let Some(prev_order) = last_order {
                if order < prev_order {
                    ctx.report(Diagnostic {
                        rule_name: "jest/prefer-hooks-in-order".to_owned(),
                        message: format!(
                            "`{callee_name}` should be placed before `{}` in the describe block",
                            HOOK_ORDER.get(prev_order).copied().unwrap_or("unknown")
                        ),
                        span: Span::new(inner_call.span.start, inner_call.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            last_order = Some(order);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferHooksInOrder)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_wrong_order() {
        let source = r"
describe('suite', () => {
    afterEach(() => {});
    beforeEach(() => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`beforeEach` after `afterEach` should be flagged"
        );
    }

    #[test]
    fn test_allows_correct_order() {
        let source = r"
describe('suite', () => {
    beforeAll(() => {});
    beforeEach(() => {});
    afterEach(() => {});
    afterAll(() => {});
});
";
        let diags = lint(source);
        assert!(diags.is_empty(), "correct hook order should not be flagged");
    }

    #[test]
    fn test_flags_before_all_after_after_all() {
        let source = r"
describe('suite', () => {
    afterAll(() => {});
    beforeAll(() => {});
});
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`beforeAll` after `afterAll` should be flagged"
        );
    }
}
