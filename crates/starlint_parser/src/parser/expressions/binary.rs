//! Pratt precedence climbing for binary and logical expressions.

use starlint_ast::node::{AstNode, BinaryExpressionNode, LogicalExpressionNode};
use starlint_ast::operator::{BinaryOperator, LogicalOperator};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

/// Binding power (precedence) for Pratt parsing.
/// Higher values bind tighter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(dead_code)]
pub(in crate::parser) enum Precedence {
    /// Lowest — sequence expression (`,`).
    None = 0,
    /// Assignment (`=`, `+=`, etc.).
    Assignment = 1,
    /// Conditional (`? :`).
    Conditional = 2,
    /// Nullish coalescing (`??`).
    NullishCoalescing = 3,
    /// Logical OR (`||`).
    LogicalOr = 4,
    /// Logical AND (`&&`).
    LogicalAnd = 5,
    /// Bitwise OR (`|`).
    BitwiseOr = 6,
    /// Bitwise XOR (`^`).
    BitwiseXor = 7,
    /// Bitwise AND (`&`).
    BitwiseAnd = 8,
    /// Equality (`==`, `!=`, `===`, `!==`).
    Equality = 9,
    /// Relational (`<`, `>`, `<=`, `>=`, `in`, `instanceof`).
    Relational = 10,
    /// Bitwise shift (`<<`, `>>`, `>>>`).
    Shift = 11,
    /// Additive (`+`, `-`).
    Additive = 12,
    /// Multiplicative (`*`, `/`, `%`).
    Multiplicative = 13,
    /// Exponentiation (`**`).
    Exponentiation = 14,
    /// Unary prefix (`!`, `~`, `typeof`, `void`, `delete`, `+`, `-`, `++`, `--`).
    Unary = 15,
    /// Update postfix (`++`, `--`).
    Update = 16,
    /// Call and member access.
    Call = 17,
}

/// Get the precedence and operator for a binary/logical token.
pub(in crate::parser) const fn infix_precedence(kind: TokenKind) -> Option<Precedence> {
    match kind {
        // Logical
        TokenKind::PipePipe => Some(Precedence::LogicalOr),
        TokenKind::AmpAmp => Some(Precedence::LogicalAnd),
        TokenKind::QuestionQuestion => Some(Precedence::NullishCoalescing),
        // Bitwise
        TokenKind::Pipe => Some(Precedence::BitwiseOr),
        TokenKind::Caret => Some(Precedence::BitwiseXor),
        TokenKind::Amp => Some(Precedence::BitwiseAnd),
        // Equality
        TokenKind::EqEq | TokenKind::NotEq | TokenKind::EqEqEq | TokenKind::NotEqEq => {
            Some(Precedence::Equality)
        }
        // Relational
        TokenKind::LAngle
        | TokenKind::RAngle
        | TokenKind::LessEq
        | TokenKind::GreaterEq
        | TokenKind::In
        | TokenKind::Instanceof => Some(Precedence::Relational),
        // Shift
        TokenKind::LessLess | TokenKind::GreaterGreater | TokenKind::GreaterGreaterGreater => {
            Some(Precedence::Shift)
        }
        // Additive
        TokenKind::Plus | TokenKind::Minus => Some(Precedence::Additive),
        // Multiplicative
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(Precedence::Multiplicative),
        // Exponentiation
        TokenKind::StarStar => Some(Precedence::Exponentiation),
        _ => None,
    }
}

/// Map a token to a binary operator.
pub(in crate::parser) const fn token_to_binary_op(kind: TokenKind) -> Option<BinaryOperator> {
    match kind {
        TokenKind::Plus => Some(BinaryOperator::Addition),
        TokenKind::Minus => Some(BinaryOperator::Subtraction),
        TokenKind::Star => Some(BinaryOperator::Multiplication),
        TokenKind::Slash => Some(BinaryOperator::Division),
        TokenKind::Percent => Some(BinaryOperator::Remainder),
        TokenKind::StarStar => Some(BinaryOperator::Exponential),
        TokenKind::EqEq => Some(BinaryOperator::Equality),
        TokenKind::NotEq => Some(BinaryOperator::Inequality),
        TokenKind::EqEqEq => Some(BinaryOperator::StrictEquality),
        TokenKind::NotEqEq => Some(BinaryOperator::StrictInequality),
        TokenKind::LAngle => Some(BinaryOperator::LessThan),
        TokenKind::RAngle => Some(BinaryOperator::GreaterThan),
        TokenKind::LessEq => Some(BinaryOperator::LessEqualThan),
        TokenKind::GreaterEq => Some(BinaryOperator::GreaterEqualThan),
        TokenKind::LessLess => Some(BinaryOperator::ShiftLeft),
        TokenKind::GreaterGreater => Some(BinaryOperator::ShiftRight),
        TokenKind::GreaterGreaterGreater => Some(BinaryOperator::ShiftRightZeroFill),
        TokenKind::Pipe => Some(BinaryOperator::BitwiseOR),
        TokenKind::Caret => Some(BinaryOperator::BitwiseXOR),
        TokenKind::Amp => Some(BinaryOperator::BitwiseAnd),
        TokenKind::In => Some(BinaryOperator::In),
        TokenKind::Instanceof => Some(BinaryOperator::Instanceof),
        _ => None,
    }
}

/// Map a token to a logical operator.
pub(in crate::parser) const fn token_to_logical_op(kind: TokenKind) -> Option<LogicalOperator> {
    match kind {
        TokenKind::PipePipe => Some(LogicalOperator::Or),
        TokenKind::AmpAmp => Some(LogicalOperator::And),
        TokenKind::QuestionQuestion => Some(LogicalOperator::Coalesce),
        _ => None,
    }
}

impl Parser<'_> {
    /// Parse a binary expression using Pratt precedence climbing.
    pub(in crate::parser) fn parse_binary_expression(
        &mut self,
        parent: Option<NodeId>,
        min_prec: Precedence,
    ) -> NodeId {
        let mut left = self.parse_unary_expression(parent);

        loop {
            let Some(prec) = infix_precedence(self.cur()) else {
                break;
            };
            if prec <= min_prec {
                break;
            }

            let op_token = self.cur();
            let start = self.tree.span(left).map_or(0, |s| s.start);
            let bin_id = self.reserve(parent);
            self.bump(); // consume operator

            // Right-associative for `**`
            // Use one level below for right-associative `**`
            let right_prec = if op_token == TokenKind::StarStar {
                Precedence::Multiplicative // lower than Exponentiation, so right side can be **
            } else {
                prec
            };

            let right = self.parse_binary_expression(Some(bin_id), right_prec);
            let end = self.tree.span(right).map_or(0, |s| s.end);

            if let Some(logical_op) = token_to_logical_op(op_token) {
                self.tree.set(
                    bin_id,
                    AstNode::LogicalExpression(LogicalExpressionNode {
                        span: Span::new(start, end),
                        operator: logical_op,
                        left,
                        right,
                    }),
                );
            } else if let Some(binary_op) = token_to_binary_op(op_token) {
                self.tree.set(
                    bin_id,
                    AstNode::BinaryExpression(BinaryExpressionNode {
                        span: Span::new(start, end),
                        operator: binary_op,
                        left,
                        right,
                    }),
                );
            }

            left = bin_id;
        }

        left
    }
}
