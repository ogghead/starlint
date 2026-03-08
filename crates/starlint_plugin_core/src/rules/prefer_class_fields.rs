//! Rule: `prefer-class-fields`
//!
//! Prefer class field declarations over `this.x = literal` assignments in
//! constructors. Class fields (ES2022) are more concise and make the shape
//! of the class immediately visible.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{AssignmentOperator, MethodDefinitionKind, UnaryOperator};
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `this.x = literal` assignments in constructors that could be class fields.
#[derive(Debug)]
pub struct PreferClassFields;

impl LintRule for PreferClassFields {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-class-fields".to_owned(),
            description: "Prefer class field declarations over constructor property initialization"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        // method.value is a NodeId pointing to a Function node
        let Some(AstNode::Function(func)) = ctx.node(method.value) else {
            return;
        };

        let Some(body_id) = func.body else {
            return;
        };

        let Some(AstNode::FunctionBody(body)) = ctx.node(body_id) else {
            return;
        };

        // Collect statement IDs first to avoid borrow conflict
        let stmt_ids: Vec<NodeId> = body.statements.to_vec();

        for stmt_id in stmt_ids {
            check_this_literal_assignment(ctx, stmt_id);
        }
    }
}

/// Check if a statement is `this.x = <literal>` and report it.
fn check_this_literal_assignment(ctx: &mut LintContext<'_>, stmt_id: NodeId) {
    let Some(AstNode::ExpressionStatement(expr_stmt)) = ctx.node(stmt_id) else {
        return;
    };
    let expr_id = expr_stmt.expression;

    let Some(AstNode::AssignmentExpression(assign)) = ctx.node(expr_id) else {
        return;
    };

    // Only check plain `=` assignments
    if assign.operator != AssignmentOperator::Assign {
        return;
    }

    // Left side must be `this.something`
    let left_id = assign.left;
    let right_id = assign.right;
    let assign_span = assign.span;

    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(left_id) else {
        return;
    };

    let is_this = ctx
        .node(member.object)
        .is_some_and(|n| matches!(n, AstNode::ThisExpression(_)));
    if !is_this {
        return;
    }

    let prop_name = member.property.clone();

    // Right side must be a literal value
    if !ctx.node(right_id).is_some_and(|n| is_literal(ctx, n)) {
        return;
    }

    ctx.report(Diagnostic {
        rule_name: "prefer-class-fields".to_owned(),
        message: format!("`this.{prop_name}` assignment can be a class field declaration"),
        span: Span::new(assign_span.start, assign_span.end),
        severity: Severity::Warning,
        help: None,
        fix: None,
        labels: vec![],
    });
}

/// Check whether an `AstNode` is a literal value (number, string, boolean,
/// null, regex, template literal with no expressions, or unary minus
/// on a numeric literal like `-1`).
fn is_literal(ctx: &LintContext<'_>, node: &AstNode) -> bool {
    match node {
        AstNode::NumericLiteral(_)
        | AstNode::StringLiteral(_)
        | AstNode::BooleanLiteral(_)
        | AstNode::NullLiteral(_)
        | AstNode::RegExpLiteral(_) => true,
        AstNode::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        AstNode::UnaryExpression(unary) => {
            unary.operator == UnaryOperator::UnaryNegation
                && ctx
                    .node(unary.argument)
                    .is_some_and(|n| matches!(n, AstNode::NumericLiteral(_)))
        }
        AstNode::ArrayExpression(arr) => arr.elements.is_empty(),
        AstNode::ObjectExpression(obj) => obj.properties.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferClassFields)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_this_assign_number() {
        let diags = lint("class A { constructor() { this.x = 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "this.x = 1 in constructor should be flagged"
        );
    }

    #[test]
    fn test_flags_this_assign_string() {
        let diags = lint("class A { constructor() { this.name = 'hello'; } }");
        assert_eq!(
            diags.len(),
            1,
            "this.name = string in constructor should be flagged"
        );
    }

    #[test]
    fn test_flags_this_assign_boolean() {
        let diags = lint("class A { constructor() { this.active = true; } }");
        assert_eq!(
            diags.len(),
            1,
            "this.active = true in constructor should be flagged"
        );
    }

    #[test]
    fn test_flags_this_assign_null() {
        let diags = lint("class A { constructor() { this.data = null; } }");
        assert_eq!(
            diags.len(),
            1,
            "this.data = null in constructor should be flagged"
        );
    }

    #[test]
    fn test_allows_class_field() {
        let diags = lint("class A { x = 1; }");
        assert!(
            diags.is_empty(),
            "class field declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_parameter_assignment() {
        let diags = lint("class A { constructor(val) { this.x = val; } }");
        assert!(diags.is_empty(), "this.x = parameter should not be flagged");
    }

    #[test]
    fn test_allows_computed_assignment() {
        let diags = lint("class A { constructor() { this.x = getVal(); } }");
        assert!(
            diags.is_empty(),
            "this.x = function call should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_literal_assignments() {
        let diags = lint("class A { constructor() { this.x = 1; this.y = 2; } }");
        assert_eq!(diags.len(), 2, "both literal assignments should be flagged");
    }

    #[test]
    fn test_allows_non_constructor_method() {
        let diags = lint("class A { init() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "assignment in non-constructor method should not be flagged"
        );
    }

    #[test]
    fn test_flags_negative_number() {
        let diags = lint("class A { constructor() { this.x = -1; } }");
        assert_eq!(
            diags.len(),
            1,
            "this.x = -1 in constructor should be flagged"
        );
    }
}
