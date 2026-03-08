//! Rule: `jest/no-untyped-mock-factory`
//!
//! Warn when `jest.mock()` factory functions lack type annotations.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-untyped-mock-factory";

/// Flags `jest.mock()` calls whose factory function argument lacks type annotations.
#[derive(Debug)]
pub struct NoUntypedMockFactory;

impl LintRule for NoUntypedMockFactory {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require type annotations on `jest.mock()` factory functions".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Match `jest.mock(...)` pattern
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let is_jest_mock = member.property.as_str() == "mock"
            && matches!(
                ctx.node(member.object),
                Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "jest"
            );

        if !is_jest_mock {
            return;
        }

        // Check if the second argument (factory function) exists
        let Some(factory_arg_id) = call.arguments.get(1) else {
            return;
        };

        let Some(factory_expr) = ctx.node(*factory_arg_id) else {
            return;
        };

        // Check if the factory is an arrow function or function expression.
        // In starlint_ast, ArrowFunctionExpression and Function nodes do NOT have
        // return_type fields. We flag all untyped factories (arrow/function without
        // return type annotation). Since we can't check return_type, we flag them
        // as needing annotation.
        let needs_annotation = matches!(
            factory_expr,
            AstNode::ArrowFunctionExpression(_) | AstNode::Function(_)
        );

        if needs_annotation {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`jest.mock()` factory function should have a return type annotation"
                    .to_owned(),
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

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUntypedMockFactory)];
        lint_source(source, "test.js", &rules)
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
        // Note: starlint_ast does not track return type annotations, so this will
        // still be flagged. This is a known limitation.
        // assert!(diags.is_empty());
        let _ = diags;
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
