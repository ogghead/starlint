//! Expression parsing with Pratt (precedence climbing) for binary operators.
//!
//! Split into submodules by concern:
//! - `assignment` — assignment, conditional, and ternary expressions
//! - `binary` — Pratt precedence climbing for binary/logical operators
//! - `literal` — array, object, template, and regex literals (plus utility fns)
//! - `postfix` — postfix update, call, member access, optional chaining, new
//! - `primary` — primary/atom expressions (identifiers, literals, parens, arrows)
//! - `unary` — unary prefix, await, prefix update expressions

pub(in crate::parser) mod assignment;
pub(in crate::parser) mod binary;
pub(in crate::parser) mod literal;
pub(in crate::parser) mod postfix;
pub(in crate::parser) mod primary;
pub(in crate::parser) mod unary;

use starlint_ast::node::{AstNode, SequenceExpressionNode};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::Parser;

impl Parser<'_> {
    /// Parse an expression (including comma/sequence expressions).
    pub(crate) fn parse_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let first = self.parse_assignment_expression(parent);
        if !self.at(TokenKind::Comma) {
            return first;
        }
        // Sequence expression
        let start = self.tree.span(first).map_or(0, |s| s.start);
        let seq_id = self.reserve(parent);
        let mut exprs = vec![first];
        // Re-parent first expression to the sequence node
        while self.eat(TokenKind::Comma) {
            let expr = self.parse_assignment_expression(Some(seq_id));
            exprs.push(expr);
        }
        let end = self
            .tree
            .span(*exprs.last().unwrap_or(&first))
            .map_or(0, |s| s.end);
        self.tree.set(
            seq_id,
            AstNode::SequenceExpression(SequenceExpressionNode {
                span: Span::new(start, end),
                expressions: exprs.into_boxed_slice(),
            }),
        );
        seq_id
    }
}
