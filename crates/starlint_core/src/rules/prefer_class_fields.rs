//! Rule: `prefer-class-fields`
//!
//! Prefer class field declarations over `this.x = literal` assignments in
//! constructors. Class fields (ES2022) are more concise and make the shape
//! of the class immediately visible.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, Expression, MethodDefinitionKind, Statement,
};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this.x = literal` assignments in constructors that could be class fields.
#[derive(Debug)]
pub struct PreferClassFields;

impl NativeRule for PreferClassFields {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-class-fields".to_owned(),
            description: "Prefer class field declarations over constructor property initialization"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::MethodDefinition])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::MethodDefinition(method) = kind else {
            return;
        };

        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        let Some(body) = &method.value.body else {
            return;
        };

        for stmt in &body.statements {
            check_this_literal_assignment(stmt, ctx);
        }
    }
}

/// Check if a statement is `this.x = <literal>` and report it.
fn check_this_literal_assignment(stmt: &Statement<'_>, ctx: &mut NativeLintContext<'_>) {
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return;
    };

    let Expression::AssignmentExpression(assign) = &expr_stmt.expression else {
        return;
    };

    // Only check plain `=` assignments
    if assign.operator != AssignmentOperator::Assign {
        return;
    }

    // Left side must be `this.something`
    let AssignmentTarget::StaticMemberExpression(member) = &assign.left else {
        return;
    };

    if !matches!(&member.object, Expression::ThisExpression(_)) {
        return;
    }

    // Right side must be a literal value
    if !is_literal(&assign.right) {
        return;
    }

    let prop_name = member.property.name.as_str();

    ctx.report(Diagnostic {
        rule_name: "prefer-class-fields".to_owned(),
        message: format!("`this.{prop_name}` assignment can be a class field declaration"),
        span: Span::new(assign.span.start, assign.span.end),
        severity: Severity::Warning,
        help: None,
        fix: None,
        labels: vec![],
    });
}

/// Check whether an expression is a literal value (number, string, boolean,
/// null, bigint, regex, template literal with no expressions, or unary minus
/// on a numeric literal like `-1`).
fn is_literal(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,
        Expression::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, oxc_ast::ast::UnaryOperator::UnaryNegation)
                && matches!(&unary.argument, Expression::NumericLiteral(_))
        }
        Expression::ArrayExpression(arr) => arr.elements.is_empty(),
        Expression::ObjectExpression(obj) => obj.properties.is_empty(),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferClassFields)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
