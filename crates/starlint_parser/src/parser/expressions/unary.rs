//! Unary, prefix update, and await expression parsing.

use starlint_ast::node::{AstNode, AwaitExpressionNode, UnaryExpressionNode, UpdateExpressionNode};
use starlint_ast::operator::{UnaryOperator, UpdateOperator};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    /// Parse a unary expression (prefix operators).
    pub(crate) fn parse_unary_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        match self.cur() {
            // Prefix unary
            TokenKind::Bang
            | TokenKind::Tilde
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Typeof
            | TokenKind::Void
            | TokenKind::Delete => {
                let start = self.start();
                let unary_id = self.reserve(parent);
                let op = match self.cur() {
                    TokenKind::Bang => UnaryOperator::LogicalNot,
                    TokenKind::Tilde => UnaryOperator::BitwiseNot,
                    TokenKind::Plus => UnaryOperator::UnaryPlus,
                    TokenKind::Minus => UnaryOperator::UnaryNegation,
                    TokenKind::Typeof => UnaryOperator::Typeof,
                    TokenKind::Void => UnaryOperator::Void,
                    TokenKind::Delete => UnaryOperator::Delete,
                    _ => UnaryOperator::LogicalNot, // unreachable
                };
                self.bump();
                let argument = self.parse_unary_expression(Some(unary_id));
                let end = self.tree.span(argument).map_or(0, |s| s.end);
                self.tree.set(
                    unary_id,
                    AstNode::UnaryExpression(UnaryExpressionNode {
                        span: Span::new(start, end),
                        operator: op,
                        argument,
                    }),
                );
                unary_id
            }
            // Prefix update (`++x`, `--x`)
            TokenKind::PlusPlus | TokenKind::MinusMinus => {
                let start = self.start();
                let update_id = self.reserve(parent);
                let op = if self.cur() == TokenKind::PlusPlus {
                    UpdateOperator::Increment
                } else {
                    UpdateOperator::Decrement
                };
                self.bump();
                let argument = self.parse_unary_expression(Some(update_id));
                let end = self.tree.span(argument).map_or(0, |s| s.end);
                self.tree.set(
                    update_id,
                    AstNode::UpdateExpression(UpdateExpressionNode {
                        span: Span::new(start, end),
                        operator: op,
                        prefix: true,
                        argument,
                    }),
                );
                update_id
            }
            // `await` expression
            TokenKind::Await => {
                let start = self.start();
                let await_id = self.reserve(parent);
                self.bump();
                let argument = self.parse_unary_expression(Some(await_id));
                let end = self.tree.span(argument).map_or(0, |s| s.end);
                self.tree.set(
                    await_id,
                    AstNode::AwaitExpression(AwaitExpressionNode {
                        span: Span::new(start, end),
                        argument,
                    }),
                );
                await_id
            }
            _ => self.parse_update_expression(parent),
        }
    }
}
