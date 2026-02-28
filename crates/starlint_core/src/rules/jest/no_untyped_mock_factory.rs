//! Rule: `jest/no-untyped-mock-factory`
//!
//! Warn when `jest.mock()` factory functions lack type annotations.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-untyped-mock-factory";

/// Flags `jest.mock()` calls whose factory function argument lacks type annotations.
#[derive(Debug)]
pub struct NoUntypedMockFactory;

impl NativeRule for NoUntypedMockFactory {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require type annotations on `jest.mock()` factory functions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Match `jest.mock(...)` pattern
        let is_jest_mock = match &call.callee {
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "mock"
                    && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "jest")
            }
            _ => false,
        };

        if !is_jest_mock {
            return;
        }

        // Check if the second argument (factory function) exists
        let Some(factory_arg) = call.arguments.get(1) else {
            return;
        };

        let Some(factory_expr) = factory_arg.as_expression() else {
            return;
        };

        // Check if the factory is an arrow function or function expression without return type
        let needs_annotation = match factory_expr {
            Expression::ArrowFunctionExpression(arrow) => arrow.return_type.is_none(),
            Expression::FunctionExpression(func) => func.return_type.is_none(),
            _ => false,
        };

        if needs_annotation {
            ctx.report_warning(
                RULE_NAME,
                "`jest.mock()` factory function should have a return type annotation",
                Span::new(call.span.start, call.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUntypedMockFactory)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_untyped_arrow_factory() {
        let source = r"jest.mock('./module', () => ({ fn: jest.fn() }));";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "untyped arrow factory in `jest.mock()` should be flagged"
        );
    }

    #[test]
    fn test_allows_typed_arrow_factory() {
        let source = r"jest.mock('./module', (): MockedModule => ({ fn: jest.fn() }));";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "typed arrow factory in `jest.mock()` should not be flagged"
        );
    }

    #[test]
    fn test_allows_mock_without_factory() {
        let source = r"jest.mock('./module');";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`jest.mock()` without factory should not be flagged"
        );
    }
}
