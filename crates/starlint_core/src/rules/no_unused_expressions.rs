//! Rule: `no-unused-expressions`
//!
//! Disallow expressions that have no side effects and whose value is
//! not used. Such expressions are likely mistakes or dead code.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UnaryOperator;
use starlint_ast::types::NodeId;

/// Flags expression statements with no side effects.
#[derive(Debug)]
pub struct NoUnusedExpressions;

impl LintRule for NoUnusedExpressions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-expressions".to_owned(),
            description: "Disallow unused expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExpressionStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ExpressionStatement(stmt) = node else {
            return;
        };

        // Allow directive-like string literals ("use strict", etc.)
        if matches!(ctx.node(stmt.expression), Some(AstNode::StringLiteral(_))) {
            return;
        }

        if is_unused_expression(stmt.expression, ctx) {
            let span = Span::new(stmt.span.start, stmt.span.end);
            let fix = FixBuilder::new("Remove unused expression", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "no-unused-expressions".to_owned(),
                message: "Expected an assignment or function call and instead saw an expression"
                    .to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is "unused" — has no side effects and its
/// value is discarded.
fn is_unused_expression(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(expr) = ctx.node(expr_id) else {
        return false;
    };
    match expr {
        // Literals, identifiers, binary/logical expressions, and template
        // literals are always unused as expression statements
        AstNode::NumericLiteral(_)
        | AstNode::BooleanLiteral(_)
        | AstNode::NullLiteral(_)
        | AstNode::RegExpLiteral(_)
        | AstNode::IdentifierReference(_)
        | AstNode::BinaryExpression(_)
        | AstNode::LogicalExpression(_)
        | AstNode::TemplateLiteral(_) => true,

        // Unary expressions that are not void/delete/typeof are unused
        // (void, delete, typeof have side effects or are intentional)
        AstNode::UnaryExpression(unary) => !matches!(
            unary.operator,
            UnaryOperator::Void | UnaryOperator::Delete | UnaryOperator::Typeof
        ),

        // Ternary/conditional — unused if both branches are unused
        AstNode::ConditionalExpression(cond) => {
            is_unused_expression(cond.consequent, ctx) && is_unused_expression(cond.alternate, ctx)
        }

        // Sequence expressions — check the last expression
        AstNode::SequenceExpression(seq) => seq
            .expressions
            .last()
            .is_some_and(|last_id| is_unused_expression(*last_id, ctx)),

        // These have side effects and are not unused:
        // AssignmentExpression, CallExpression, NewExpression,
        // UpdateExpression (++/--), YieldExpression, AwaitExpression,
        // TaggedTemplateExpression (calls a function)
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnusedExpressions)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_standalone_literal() {
        let diags = lint("42;");
        assert_eq!(diags.len(), 1, "standalone number should be flagged");
    }

    #[test]
    fn test_flags_standalone_identifier() {
        let diags = lint("foo;");
        assert_eq!(diags.len(), 1, "standalone identifier should be flagged");
    }

    #[test]
    fn test_flags_binary_expression() {
        let diags = lint("a + b;");
        assert_eq!(diags.len(), 1, "binary expression should be flagged");
    }

    #[test]
    fn test_allows_function_call() {
        let diags = lint("foo();");
        assert!(diags.is_empty(), "function call should not be flagged");
    }

    #[test]
    fn test_allows_assignment() {
        let diags = lint("x = 1;");
        assert!(diags.is_empty(), "assignment should not be flagged");
    }

    #[test]
    fn test_allows_increment() {
        let diags = lint("i++;");
        assert!(diags.is_empty(), "increment should not be flagged");
    }

    #[test]
    fn test_allows_use_strict() {
        let diags = lint("\"use strict\";");
        assert!(diags.is_empty(), "directive strings should not be flagged");
    }

    #[test]
    fn test_allows_delete() {
        let diags = lint("delete obj.prop;");
        assert!(diags.is_empty(), "delete should not be flagged");
    }

    #[test]
    fn test_allows_await() {
        let diags = lint("async function foo() { await bar; }");
        assert!(diags.is_empty(), "await should not be flagged");
    }
}
