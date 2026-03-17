//! Assignment, conditional, and ternary expression parsing.

use starlint_ast::node::{AssignmentExpressionNode, AstNode, ConditionalExpressionNode};
use starlint_ast::operator::AssignmentOperator;
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;
use super::binary::Precedence;

/// Map a token to an assignment operator.
const fn token_to_assignment_op(kind: TokenKind) -> Option<AssignmentOperator> {
    match kind {
        TokenKind::Eq => Some(AssignmentOperator::Assign),
        TokenKind::PlusEq => Some(AssignmentOperator::Addition),
        TokenKind::MinusEq => Some(AssignmentOperator::Subtraction),
        TokenKind::StarEq => Some(AssignmentOperator::Multiplication),
        TokenKind::SlashEq => Some(AssignmentOperator::Division),
        TokenKind::PercentEq => Some(AssignmentOperator::Remainder),
        TokenKind::StarStarEq => Some(AssignmentOperator::Exponential),
        TokenKind::LessLessEq => Some(AssignmentOperator::ShiftLeft),
        TokenKind::GreaterGreaterEq => Some(AssignmentOperator::ShiftRight),
        TokenKind::GreaterGreaterGreaterEq => Some(AssignmentOperator::ShiftRightZeroFill),
        TokenKind::AmpEq => Some(AssignmentOperator::BitwiseAnd),
        TokenKind::PipeEq => Some(AssignmentOperator::BitwiseOR),
        TokenKind::CaretEq => Some(AssignmentOperator::BitwiseXOR),
        TokenKind::PipePipeEq => Some(AssignmentOperator::LogicalOr),
        TokenKind::AmpAmpEq => Some(AssignmentOperator::LogicalAnd),
        TokenKind::QuestionQuestionEq => Some(AssignmentOperator::LogicalNullish),
        _ => None,
    }
}

impl Parser<'_> {
    /// Parse an assignment expression (right-associative).
    pub(crate) fn parse_assignment_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let left = self.parse_conditional_expression(parent);

        if let Some(op) = token_to_assignment_op(self.cur()) {
            let start = self.tree.span(left).map_or(0, |s| s.start);
            let assign_id = self.reserve(parent);
            self.bump(); // consume operator
            let right = self.parse_assignment_expression(Some(assign_id));
            let end = self.tree.span(right).map_or(0, |s| s.end);
            self.tree.set(
                assign_id,
                AstNode::AssignmentExpression(AssignmentExpressionNode {
                    span: Span::new(start, end),
                    operator: op,
                    left,
                    right,
                }),
            );
            return assign_id;
        }

        left
    }

    /// Parse a conditional (ternary) expression.
    fn parse_conditional_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let mut test = self.parse_binary_expression(parent, Precedence::None);

        // TypeScript postfix operators: `as Type`, `!` (non-null assertion)
        if self.options.typescript {
            test = self.parse_ts_postfix_expressions(test, parent);
        }

        if !self.at(TokenKind::Question) {
            return test;
        }

        let start = self.tree.span(test).map_or(0, |s| s.start);
        let cond_id = self.reserve(parent);
        self.bump(); // consume `?`
        let consequent = self.parse_assignment_expression(Some(cond_id));
        let _ = self.expect(TokenKind::Colon);
        let alternate = self.parse_assignment_expression(Some(cond_id));
        let end = self.tree.span(alternate).map_or(0, |s| s.end);
        self.tree.set(
            cond_id,
            AstNode::ConditionalExpression(ConditionalExpressionNode {
                span: Span::new(start, end),
                test,
                consequent,
                alternate,
            }),
        );
        cond_id
    }
}
