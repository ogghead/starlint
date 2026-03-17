//! Shared AST utility functions for lint rules.
//!
//! Common predicates and helpers extracted from across plugin crates
//! to eliminate duplication and provide a discoverable API for rule authors.

use starlint_ast::node::AstNode;
use starlint_ast::operator::BinaryOperator;

/// Check whether a node is any kind of literal.
///
/// Returns `true` for string, numeric, boolean, null, and regex literals.
#[must_use]
pub const fn is_literal(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::StringLiteral(_)
            | AstNode::NumericLiteral(_)
            | AstNode::BooleanLiteral(_)
            | AstNode::NullLiteral(_)
            | AstNode::RegExpLiteral(_)
    )
}

/// Check whether a node is an object or array literal.
#[must_use]
pub const fn is_object_or_array_literal(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::ObjectExpression(_) | AstNode::ArrayExpression(_)
    )
}

/// Check whether a node is an identifier reference with the given name.
#[must_use]
pub fn is_identifier_with_name(node: &AstNode, name: &str) -> bool {
    matches!(node, AstNode::IdentifierReference(id) if id.name.as_str() == name)
}

/// Check whether a node is the `undefined` identifier.
#[must_use]
pub fn is_undefined_identifier(node: &AstNode) -> bool {
    is_identifier_with_name(node, "undefined")
}

/// Check whether a node is a `null` literal.
#[must_use]
pub const fn is_null_literal(node: &AstNode) -> bool {
    matches!(node, AstNode::NullLiteral(_))
}

/// Check whether a node represents a nullish value (`null` or `undefined`).
#[must_use]
pub fn is_nullish(node: &AstNode) -> bool {
    is_null_literal(node) || is_undefined_identifier(node)
}

/// Check whether a node is a numeric literal with the given value.
#[must_use]
pub fn is_numeric_literal_value(node: &AstNode, value: f64) -> bool {
    matches!(node, AstNode::NumericLiteral(lit) if (lit.value - value).abs() < f64::EPSILON)
}

/// Check whether a node is the numeric literal `0`.
#[must_use]
pub fn is_zero(node: &AstNode) -> bool {
    is_numeric_literal_value(node, 0.0)
}

/// Check whether a node is the numeric literal `1`.
#[must_use]
pub fn is_one(node: &AstNode) -> bool {
    is_numeric_literal_value(node, 1.0)
}

/// Check whether a node is the unary expression `-1`.
///
/// Matches the pattern `UnaryExpression { operator: UnaryNegation, argument: NumericLiteral(1) }`.
/// Requires access to the tree to resolve the argument node.
#[must_use]
pub fn is_negative_one(node: &AstNode, tree: &starlint_ast::tree::AstTree) -> bool {
    let AstNode::UnaryExpression(expr) = node else {
        return false;
    };
    if expr.operator != starlint_ast::operator::UnaryOperator::UnaryNegation {
        return false;
    }
    tree.get(expr.argument).is_some_and(is_one)
}

/// Check whether a binary operator is a bitwise operator.
#[must_use]
pub const fn is_bitwise_operator(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
    )
}

/// Check whether a binary operator is a comparison operator.
#[must_use]
pub const fn is_comparison_operator(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
    )
}

/// Check whether a binary operator is a strict equality check (`===` or `!==`).
#[must_use]
pub const fn is_strict_equality(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
    )
}

/// Check whether a node is a function boundary (function, arrow function, or class method).
///
/// Useful for rules that need to track scope boundaries without full scope analysis.
#[must_use]
pub const fn is_function_boundary(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::Function(_) | AstNode::ArrowFunctionExpression(_)
    )
}

/// Check whether a node is a loop statement.
#[must_use]
pub const fn is_loop(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::ForStatement(_)
            | AstNode::ForInStatement(_)
            | AstNode::ForOfStatement(_)
            | AstNode::WhileStatement(_)
            | AstNode::DoWhileStatement(_)
    )
}

/// Check whether a node is a `StaticMemberExpression` with the given property name.
#[must_use]
pub fn is_static_member_with_property(node: &AstNode, property: &str) -> bool {
    matches!(
        node,
        AstNode::StaticMemberExpression(member) if member.property.as_str() == property
    )
}

/// Extract the identifier name from a node if it is an `IdentifierReference`.
#[must_use]
pub fn get_identifier_name(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::IdentifierReference(id) => Some(id.name.as_str()),
        AstNode::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

/// Extract the string value from a `StringLiteral` node.
#[must_use]
pub fn get_string_literal_value(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use starlint_ast::node::{
        AstNode, BinaryExpressionNode, BooleanLiteralNode, IdentifierReferenceNode,
        NullLiteralNode, NumericLiteralNode, StringLiteralNode,
    };
    use starlint_ast::operator::BinaryOperator;
    use starlint_ast::types::{NodeId, Span};

    use super::*;

    #[test]
    fn test_is_literal() {
        let string = AstNode::StringLiteral(StringLiteralNode {
            span: Span::EMPTY,
            value: "hello".into(),
        });
        assert!(is_literal(&string), "string literal should be a literal");

        let num = AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::EMPTY,
            value: 42.0,
            raw: "42".into(),
        });
        assert!(is_literal(&num), "numeric literal should be a literal");

        let bool_lit = AstNode::BooleanLiteral(BooleanLiteralNode {
            span: Span::EMPTY,
            value: true,
        });
        assert!(is_literal(&bool_lit), "boolean literal should be a literal");

        let null_lit = AstNode::NullLiteral(NullLiteralNode { span: Span::EMPTY });
        assert!(is_literal(&null_lit), "null literal should be a literal");
    }

    #[test]
    fn test_is_identifier_with_name() {
        let id = AstNode::IdentifierReference(IdentifierReferenceNode {
            span: Span::EMPTY,
            name: "undefined".into(),
        });
        assert!(is_identifier_with_name(&id, "undefined"));
        assert!(!is_identifier_with_name(&id, "null"));
    }

    #[test]
    fn test_is_undefined_identifier() {
        let undef = AstNode::IdentifierReference(IdentifierReferenceNode {
            span: Span::EMPTY,
            name: "undefined".into(),
        });
        assert!(is_undefined_identifier(&undef));

        let other = AstNode::IdentifierReference(IdentifierReferenceNode {
            span: Span::EMPTY,
            name: "foo".into(),
        });
        assert!(!is_undefined_identifier(&other));
    }

    #[test]
    fn test_is_nullish() {
        let null_lit = AstNode::NullLiteral(NullLiteralNode { span: Span::EMPTY });
        assert!(is_nullish(&null_lit));

        let undef = AstNode::IdentifierReference(IdentifierReferenceNode {
            span: Span::EMPTY,
            name: "undefined".into(),
        });
        assert!(is_nullish(&undef));

        let num = AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::EMPTY,
            value: 0.0,
            raw: "0".into(),
        });
        assert!(!is_nullish(&num));
    }

    #[test]
    fn test_is_zero_and_one() {
        let zero = AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::EMPTY,
            value: 0.0,
            raw: "0".into(),
        });
        assert!(is_zero(&zero));
        assert!(!is_one(&zero));

        let one = AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::EMPTY,
            value: 1.0,
            raw: "1".into(),
        });
        assert!(is_one(&one));
        assert!(!is_zero(&one));
    }

    #[test]
    fn test_is_bitwise_operator() {
        assert!(is_bitwise_operator(BinaryOperator::BitwiseAnd));
        assert!(is_bitwise_operator(BinaryOperator::ShiftLeft));
        assert!(!is_bitwise_operator(BinaryOperator::Addition));
        assert!(!is_bitwise_operator(BinaryOperator::StrictEquality));
    }

    #[test]
    fn test_is_comparison_operator() {
        assert!(is_comparison_operator(BinaryOperator::StrictEquality));
        assert!(is_comparison_operator(BinaryOperator::LessThan));
        assert!(!is_comparison_operator(BinaryOperator::Addition));
    }

    #[test]
    fn test_is_strict_equality() {
        assert!(is_strict_equality(BinaryOperator::StrictEquality));
        assert!(is_strict_equality(BinaryOperator::StrictInequality));
        assert!(!is_strict_equality(BinaryOperator::Equality));
    }

    #[test]
    fn test_is_function_boundary() {
        let binary = AstNode::BinaryExpression(BinaryExpressionNode {
            span: Span::EMPTY,
            operator: BinaryOperator::Addition,
            left: NodeId::NONE,
            right: NodeId::NONE,
        });
        assert!(!is_function_boundary(&binary));
    }

    #[test]
    fn test_get_identifier_name() {
        let id = AstNode::IdentifierReference(IdentifierReferenceNode {
            span: Span::EMPTY,
            name: "foo".into(),
        });
        assert_eq!(get_identifier_name(&id), Some("foo"));

        let num = AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::EMPTY,
            value: 42.0,
            raw: "42".into(),
        });
        assert_eq!(get_identifier_name(&num), None);
    }

    #[test]
    fn test_get_string_literal_value() {
        let s = AstNode::StringLiteral(StringLiteralNode {
            span: Span::EMPTY,
            value: "hello".into(),
        });
        assert_eq!(get_string_literal_value(&s), Some("hello"));

        let num = AstNode::NumericLiteral(NumericLiteralNode {
            span: Span::EMPTY,
            value: 42.0,
            raw: "42".into(),
        });
        assert_eq!(get_string_literal_value(&num), None);
    }
}
