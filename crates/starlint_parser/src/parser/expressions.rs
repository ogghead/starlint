//! Expression parsing with Pratt (precedence climbing) for binary operators.

use starlint_ast::node::{
    ArrayExpressionNode, ArrowFunctionExpressionNode, AssignmentExpressionNode, AstNode,
    AwaitExpressionNode, BinaryExpressionNode, BooleanLiteralNode, CallExpressionNode,
    ChainExpressionNode, ComputedMemberExpressionNode, ConditionalExpressionNode,
    IdentifierReferenceNode, LogicalExpressionNode, NewExpressionNode, NullLiteralNode,
    NumericLiteralNode, ObjectExpressionNode, ObjectPropertyNode, RegExpLiteralNode,
    SequenceExpressionNode, SpreadElementNode, StaticMemberExpressionNode, StringLiteralNode,
    TSNonNullExpressionNode, TaggedTemplateExpressionNode, TemplateLiteralNode, ThisExpressionNode,
    UnaryExpressionNode, UnknownNode, UpdateExpressionNode,
};
use starlint_ast::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, PropertyKind, UnaryOperator,
    UpdateOperator,
};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::Parser;

/// Binding power (precedence) for Pratt parsing.
/// Higher values bind tighter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(dead_code)]
enum Precedence {
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
const fn infix_precedence(kind: TokenKind) -> Option<Precedence> {
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
const fn token_to_binary_op(kind: TokenKind) -> Option<BinaryOperator> {
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
const fn token_to_logical_op(kind: TokenKind) -> Option<LogicalOperator> {
    match kind {
        TokenKind::PipePipe => Some(LogicalOperator::Or),
        TokenKind::AmpAmp => Some(LogicalOperator::And),
        TokenKind::QuestionQuestion => Some(LogicalOperator::Coalesce),
        _ => None,
    }
}

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

    /// Parse a binary expression using Pratt precedence climbing.
    fn parse_binary_expression(&mut self, parent: Option<NodeId>, min_prec: Precedence) -> NodeId {
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

    /// Parse a unary expression (prefix operators).
    fn parse_unary_expression(&mut self, parent: Option<NodeId>) -> NodeId {
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

    /// Parse a postfix update expression (`x++`, `x--`).
    fn parse_update_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let expr = self.parse_left_hand_side_expression(parent);

        // Postfix: no line terminator between operand and `++`/`--`
        if !self.has_preceding_line_break()
            && (self.at(TokenKind::PlusPlus) || self.at(TokenKind::MinusMinus))
        {
            let start = self.tree.span(expr).map_or(0, |s| s.start);
            let update_id = self.reserve(parent);
            let op = if self.cur() == TokenKind::PlusPlus {
                UpdateOperator::Increment
            } else {
                UpdateOperator::Decrement
            };
            let tok = self.bump();
            self.tree.set(
                update_id,
                AstNode::UpdateExpression(UpdateExpressionNode {
                    span: Span::new(start, tok.end),
                    operator: op,
                    prefix: false,
                    argument: expr,
                }),
            );
            return update_id;
        }

        expr
    }

    /// Parse a left-hand-side expression (call, member access, `new`).
    pub(crate) fn parse_left_hand_side_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let mut expr = if self.at(TokenKind::New) {
            self.parse_new_expression(parent)
        } else {
            self.parse_primary_expression(parent)
        };

        // Call/member chain
        loop {
            match self.cur() {
                // `expr.prop`
                TokenKind::Dot => {
                    let start = self.tree.span(expr).map_or(0, |s| s.start);
                    let member_id = self.reserve(parent);
                    self.bump(); // `.`
                    let prop_name = self.cur_text().to_owned();
                    let end = self.current.end;
                    self.bump(); // property name
                    self.tree.set(
                        member_id,
                        AstNode::StaticMemberExpression(StaticMemberExpressionNode {
                            span: Span::new(start, end),
                            object: expr,
                            property: prop_name,
                            optional: false,
                        }),
                    );
                    expr = member_id;
                }
                // `expr[computed]`
                TokenKind::LBracket => {
                    let start = self.tree.span(expr).map_or(0, |s| s.start);
                    let member_id = self.reserve(parent);
                    self.bump(); // `[`
                    let prop = self.parse_expression(Some(member_id));
                    let end = self.current.end;
                    let _ = self.expect(TokenKind::RBracket);
                    self.tree.set(
                        member_id,
                        AstNode::ComputedMemberExpression(ComputedMemberExpressionNode {
                            span: Span::new(start, end),
                            object: expr,
                            expression: prop,
                            optional: false,
                        }),
                    );
                    expr = member_id;
                }
                // `expr(args)`
                TokenKind::LParen => {
                    let start = self.tree.span(expr).map_or(0, |s| s.start);
                    let call_id = self.reserve(parent);
                    let args = self.parse_arguments(Some(call_id));
                    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                    let end = self.prev_end;
                    self.tree.set(
                        call_id,
                        AstNode::CallExpression(CallExpressionNode {
                            span: Span::new(start, end),
                            callee: expr,
                            arguments: args.into_boxed_slice(),
                            optional: false,
                        }),
                    );
                    expr = call_id;
                }
                // Optional chaining `expr?.prop`, `expr?.[computed]`, `expr?.(args)`
                TokenKind::QuestionDot => {
                    let start = self.tree.span(expr).map_or(0, |s| s.start);
                    let chain_id = self.reserve(parent);
                    self.bump(); // `?.`

                    let inner = if self.at(TokenKind::LParen) {
                        let inner_call_id = self.reserve(Some(chain_id));
                        let args = self.parse_arguments(Some(inner_call_id));
                        let end = self.prev_end;
                        self.tree.set(
                            inner_call_id,
                            AstNode::CallExpression(CallExpressionNode {
                                span: Span::new(start, end),
                                callee: expr,
                                arguments: args.into_boxed_slice(),
                                optional: true,
                            }),
                        );
                        inner_call_id
                    } else if self.at(TokenKind::LBracket) {
                        let inner_member_id = self.reserve(Some(chain_id));
                        self.bump(); // `[`
                        let prop = self.parse_expression(Some(inner_member_id));
                        let end = self.current.end;
                        let _ = self.expect(TokenKind::RBracket);
                        self.tree.set(
                            inner_member_id,
                            AstNode::ComputedMemberExpression(ComputedMemberExpressionNode {
                                span: Span::new(start, end),
                                object: expr,
                                expression: prop,
                                optional: true,
                            }),
                        );
                        inner_member_id
                    } else {
                        let inner_member_id = self.reserve(Some(chain_id));
                        let prop_name = self.cur_text().to_owned();
                        let end = self.current.end;
                        self.bump();
                        self.tree.set(
                            inner_member_id,
                            AstNode::StaticMemberExpression(StaticMemberExpressionNode {
                                span: Span::new(start, end),
                                object: expr,
                                property: prop_name,
                                optional: true,
                            }),
                        );
                        inner_member_id
                    };

                    let end = self.tree.span(inner).map_or(0, |s| s.end);
                    self.tree.set(
                        chain_id,
                        AstNode::ChainExpression(ChainExpressionNode {
                            span: Span::new(start, end),
                            expression: inner,
                        }),
                    );
                    expr = chain_id;
                }
                // TypeScript non-null assertion `expr!`
                TokenKind::Bang if self.options.typescript && !self.has_preceding_line_break() => {
                    // Only treat as non-null if not part of `!=` or `!==`
                    let next_byte = self
                        .source
                        .as_bytes()
                        .get(self.current.end as usize)
                        .copied();
                    if next_byte == Some(b'=') {
                        break;
                    }
                    let start = self.tree.span(expr).map_or(0, |s| s.start);
                    let tok = self.bump(); // `!`
                    let nn_id = self.push(
                        AstNode::TSNonNullExpression(TSNonNullExpressionNode {
                            span: Span::new(start, tok.end),
                            expression: expr,
                        }),
                        parent,
                    );
                    expr = nn_id;
                }
                // Tagged template `` expr`...` ``
                TokenKind::NoSubstitutionTemplate | TokenKind::TemplateHead => {
                    let start = self.tree.span(expr).map_or(0, |s| s.start);
                    let tagged_id = self.reserve(parent);
                    let quasi = self.parse_template_literal(Some(tagged_id));
                    let end = self.tree.span(quasi).map_or(0, |s| s.end);
                    self.tree.set(
                        tagged_id,
                        AstNode::TaggedTemplateExpression(TaggedTemplateExpressionNode {
                            span: Span::new(start, end),
                            tag: expr,
                            quasi,
                        }),
                    );
                    expr = tagged_id;
                }
                _ => break,
            }
        }

        expr
    }

    /// Parse a `new` expression.
    fn parse_new_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let new_id = self.reserve(parent);
        self.bump(); // `new`

        // `new.target` — special case
        if self.at(TokenKind::Dot) {
            // Simplified: just parse as member expression
        }

        let mut callee = if self.at(TokenKind::New) {
            self.parse_new_expression(Some(new_id))
        } else {
            self.parse_primary_expression(Some(new_id))
        };

        // Member access chain on the callee, e.g., `new Foo.Bar()`
        loop {
            match self.cur() {
                TokenKind::Dot => {
                    let member_start = self.tree.span(callee).map_or(0, |s| s.start);
                    let member_id = self.reserve(Some(new_id));
                    self.bump(); // `.`
                    let prop_name = self.cur_text().to_owned();
                    let end = self.current.end;
                    self.bump();
                    self.tree.set(
                        member_id,
                        AstNode::StaticMemberExpression(StaticMemberExpressionNode {
                            span: Span::new(member_start, end),
                            object: callee,
                            property: prop_name,
                            optional: false,
                        }),
                    );
                    callee = member_id;
                }
                TokenKind::LBracket => {
                    let member_start = self.tree.span(callee).map_or(0, |s| s.start);
                    let member_id = self.reserve(Some(new_id));
                    self.bump(); // `[`
                    let prop = self.parse_expression(Some(member_id));
                    let end = self.current.end;
                    let _ = self.expect(TokenKind::RBracket);
                    self.tree.set(
                        member_id,
                        AstNode::ComputedMemberExpression(ComputedMemberExpressionNode {
                            span: Span::new(member_start, end),
                            object: callee,
                            expression: prop,
                            optional: false,
                        }),
                    );
                    callee = member_id;
                }
                _ => break,
            }
        }

        let args = if self.at(TokenKind::LParen) {
            self.parse_arguments(Some(new_id))
        } else {
            Vec::new()
        };

        let end = if args.is_empty() {
            self.tree.span(callee).map_or(0, |s| s.end)
        } else {
            self.prev_end
        };

        self.tree.set(
            new_id,
            AstNode::NewExpression(NewExpressionNode {
                span: Span::new(start, end),
                callee,
                arguments: args.into_boxed_slice(),
            }),
        );
        new_id
    }

    /// Parse argument list `(arg1, arg2, ...)`.
    pub(crate) fn parse_arguments(&mut self, parent: Option<NodeId>) -> Vec<NodeId> {
        let _ = self.expect(TokenKind::LParen);
        let mut args = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::DotDotDot) {
                let spread_start = self.start();
                let spread_id = self.reserve(parent);
                self.bump(); // `...`
                let arg = self.parse_assignment_expression(Some(spread_id));
                let end = self.tree.span(arg).map_or(0, |s| s.end);
                self.tree.set(
                    spread_id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Span::new(spread_start, end),
                        argument: arg,
                    }),
                );
                args.push(spread_id);
            } else {
                let arg = self.parse_assignment_expression(parent);
                args.push(arg);
            }
            if !self.at(TokenKind::RParen) {
                let _ = self.expect(TokenKind::Comma);
            }
        }
        let _ = self.expect(TokenKind::RParen);
        args
    }

    /// Parse a primary expression.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse_primary_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();

        match self.cur() {
            TokenKind::Identifier
            | TokenKind::Async
            | TokenKind::From
            | TokenKind::Of
            | TokenKind::Get
            | TokenKind::Set
            | TokenKind::Let
            | TokenKind::Static
            | TokenKind::As
            | TokenKind::Type
            | TokenKind::Declare
            | TokenKind::Namespace
            | TokenKind::Module
            | TokenKind::Abstract
            | TokenKind::Readonly
            | TokenKind::Override
            | TokenKind::Any
            | TokenKind::Unknown
            | TokenKind::Never
            | TokenKind::Using
            | TokenKind::Satisfies
            | TokenKind::Implements
            | TokenKind::Interface
            | TokenKind::Package
            | TokenKind::Private
            | TokenKind::Protected
            | TokenKind::Public
            | TokenKind::Keyof
            | TokenKind::Unique
            | TokenKind::Infer
            | TokenKind::Is
            | TokenKind::Asserts => {
                // Check for arrow function: `ident =>`
                if self.cur() == TokenKind::Async && !self.has_preceding_line_break() {
                    if self.peek_next_is_function() {
                        return self.parse_function_expression(parent, true);
                    }
                    // `async (params) => body` or `async param => body`
                    let next = self.peek_next_text();
                    if next == "(" || next == "other" {
                        // Consume `async` and set flag for arrow function constructors
                        self.bump();
                        self.pending_async = true;
                        if self.at(TokenKind::LParen) {
                            // `async (params) => ...` — handled by paren-expr path
                            return self.parse_primary_expression(parent);
                        }
                        // `async ident => ...`
                        let ident_name = self.cur_text().to_owned();
                        let ident_tok = self.bump();
                        if self.at(TokenKind::Arrow) && !self.has_preceding_line_break() {
                            return self.parse_arrow_function_from_param(parent, start, ident_name);
                        }
                        // Not an arrow — treat `async` as identifier (already consumed)
                        self.pending_async = false;
                        return self.push(
                            AstNode::IdentifierReference(IdentifierReferenceNode {
                                span: Span::new(ident_tok.start, ident_tok.end),
                                name: ident_name,
                            }),
                            parent,
                        );
                    }
                }
                let name = self.cur_text().to_owned();
                let tok = self.bump();
                // Check for `ident =>`
                if self.at(TokenKind::Arrow) && !self.has_preceding_line_break() {
                    return self.parse_arrow_function_from_param(parent, start, name);
                }
                self.push(
                    AstNode::IdentifierReference(IdentifierReferenceNode {
                        span: Span::new(start, tok.end),
                        name,
                    }),
                    parent,
                )
            }
            TokenKind::Number => {
                let text = self.cur_text();
                let value = parse_number(text);
                let tok = self.bump();
                self.push(
                    AstNode::NumericLiteral(NumericLiteralNode {
                        span: Span::new(start, tok.end),
                        value,
                        raw: self.text(start, tok.end).to_owned(),
                    }),
                    parent,
                )
            }
            TokenKind::String => {
                let tok = self.bump();
                let raw = self.text(start, tok.end);
                // Strip quotes
                let value = if raw.len() >= 2 {
                    raw.get(1..raw.len().saturating_sub(1))
                        .unwrap_or_default()
                        .to_owned()
                } else {
                    String::new()
                };
                self.push(
                    AstNode::StringLiteral(StringLiteralNode {
                        span: Span::new(start, tok.end),
                        value,
                    }),
                    parent,
                )
            }
            TokenKind::True => {
                let tok = self.bump();
                self.push(
                    AstNode::BooleanLiteral(BooleanLiteralNode {
                        span: Span::new(start, tok.end),
                        value: true,
                    }),
                    parent,
                )
            }
            TokenKind::False => {
                let tok = self.bump();
                self.push(
                    AstNode::BooleanLiteral(BooleanLiteralNode {
                        span: Span::new(start, tok.end),
                        value: false,
                    }),
                    parent,
                )
            }
            TokenKind::Null => {
                let tok = self.bump();
                self.push(
                    AstNode::NullLiteral(NullLiteralNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
            TokenKind::This => {
                let tok = self.bump();
                self.push(
                    AstNode::ThisExpression(ThisExpressionNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
            TokenKind::RegExp => {
                let tok = self.bump();
                let raw = self.text(start, tok.end);
                // Parse /pattern/flags
                let (pattern, flags) = parse_regex(raw);
                self.push(
                    AstNode::RegExpLiteral(RegExpLiteralNode {
                        span: Span::new(start, tok.end),
                        pattern,
                        flags,
                    }),
                    parent,
                )
            }
            TokenKind::NoSubstitutionTemplate | TokenKind::TemplateHead => {
                self.parse_template_literal(parent)
            }
            TokenKind::LParen => {
                // Parenthesized expression (or arrow function params)
                self.parse_parenthesized_or_arrow(parent)
            }
            TokenKind::LBracket => self.parse_array_literal(parent),
            TokenKind::LBrace => self.parse_object_literal(parent),
            TokenKind::Function => self.parse_function_expression(parent, false),
            TokenKind::Class => self.parse_class_expression(parent),
            TokenKind::LAngle if self.options.jsx => self.parse_jsx_element(parent),
            TokenKind::Super => {
                let tok = self.bump();
                self.push(
                    AstNode::IdentifierReference(IdentifierReferenceNode {
                        span: Span::new(start, tok.end),
                        name: "super".to_owned(),
                    }),
                    parent,
                )
            }
            _ => {
                self.error(format!("unexpected token {:?}", self.cur()));
                let tok = self.bump();
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
        }
    }

    /// Parse a parenthesized expression or arrow function.
    fn parse_parenthesized_or_arrow(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `(`

        // Check for empty parens: `() =>`
        if self.at(TokenKind::RParen) {
            self.bump(); // `)`
            if self.at(TokenKind::Arrow) && !self.has_preceding_line_break() {
                return self.parse_arrow_function_body(parent, start, &[]);
            }
            // Empty parens not followed by `=>` — error
            self.error("unexpected `()`");
            return self.push(
                AstNode::Unknown(UnknownNode {
                    span: Span::new(start, self.prev_end),
                }),
                parent,
            );
        }

        // Could be `(expr)` or `(params) =>`
        // Parse as expression first, then check for `=>`
        if self.at(TokenKind::DotDotDot) {
            // Definitely arrow params: `(...rest) =>`
            return self.parse_arrow_function_with_rest_from_paren(parent, start);
        }

        let inner = self.parse_expression(parent);
        let _ = self.expect(TokenKind::RParen);

        // Check for arrow: `(expr) =>`
        if self.at(TokenKind::Arrow) && !self.has_preceding_line_break() {
            // Decompose SequenceExpression into individual params:
            // `(a, b) =>` parsed as SequenceExpression([a, b]) → params = [a, b]
            let params = match self.tree.get(inner) {
                Some(AstNode::SequenceExpression(seq)) => seq.expressions.to_vec(),
                _ => vec![inner],
            };
            return self.parse_arrow_function_body(parent, start, &params);
        }

        // Just a parenthesized expression — return inner directly
        inner
    }

    /// Parse an arrow function from a single identifier parameter.
    fn parse_arrow_function_from_param(
        &mut self,
        parent: Option<NodeId>,
        start: u32,
        name: String,
    ) -> NodeId {
        let is_async = std::mem::take(&mut self.pending_async);
        let arrow_id = self.reserve(parent);
        self.bump(); // `=>`

        // Create the parameter as a BindingIdentifier
        let param_id = self.push(
            AstNode::BindingIdentifier(starlint_ast::node::BindingIdentifierNode {
                span: Span::new(
                    start,
                    start.saturating_add(u32::try_from(name.len()).unwrap_or(0)),
                ),
                name,
            }),
            Some(arrow_id),
        );

        let is_expression = !self.at(TokenKind::LBrace);
        let body = self.parse_arrow_function_concise_body(Some(arrow_id));
        let end = self.tree.span(body).map_or(0, |s| s.end);

        self.tree.set(
            arrow_id,
            AstNode::ArrowFunctionExpression(ArrowFunctionExpressionNode {
                span: Span::new(start, end),
                is_async,
                expression: is_expression,
                params: Box::new([param_id]),
                body,
            }),
        );
        arrow_id
    }

    /// Parse arrow function body (from after `=>` with known params).
    fn parse_arrow_function_body(
        &mut self,
        parent: Option<NodeId>,
        start: u32,
        params: &[NodeId],
    ) -> NodeId {
        let is_async = std::mem::take(&mut self.pending_async);
        let arrow_id = self.reserve(parent);
        self.bump(); // `=>`

        let is_expression = !self.at(TokenKind::LBrace);
        let body = self.parse_arrow_function_concise_body(Some(arrow_id));
        let end = self.tree.span(body).map_or(0, |s| s.end);

        self.tree.set(
            arrow_id,
            AstNode::ArrowFunctionExpression(ArrowFunctionExpressionNode {
                span: Span::new(start, end),
                is_async,
                expression: is_expression,
                params: params.to_vec().into_boxed_slice(),
                body,
            }),
        );
        arrow_id
    }

    /// Parse arrow function with `...rest` param.
    fn parse_arrow_function_with_rest_from_paren(
        &mut self,
        parent: Option<NodeId>,
        start: u32,
    ) -> NodeId {
        let is_async = std::mem::take(&mut self.pending_async);
        let arrow_id = self.reserve(parent);
        // We're after `(` and at `...`
        self.bump(); // `...`
        let rest_name = self.cur_text().to_owned();
        let rest_start = self.start();
        let rest_tok = self.bump();
        let rest_id = self.push(
            AstNode::BindingIdentifier(starlint_ast::node::BindingIdentifierNode {
                span: Span::new(rest_start, rest_tok.end),
                name: rest_name,
            }),
            Some(arrow_id),
        );

        let _ = self.expect(TokenKind::RParen);
        let _ = self.expect(TokenKind::Arrow);

        let is_expression = !self.at(TokenKind::LBrace);
        let body = self.parse_arrow_function_concise_body(Some(arrow_id));
        let end = self.tree.span(body).map_or(0, |s| s.end);

        self.tree.set(
            arrow_id,
            AstNode::ArrowFunctionExpression(ArrowFunctionExpressionNode {
                span: Span::new(start, end),
                is_async,
                expression: is_expression,
                params: Box::new([rest_id]),
                body,
            }),
        );
        arrow_id
    }

    /// Parse the concise body of an arrow function (either expression or block).
    fn parse_arrow_function_concise_body(&mut self, parent: Option<NodeId>) -> NodeId {
        if self.at(TokenKind::LBrace) {
            self.parse_function_body(parent)
        } else {
            // Concise body: wrap expression in FunctionBody → ExpressionStatement
            // to match oxc's structure (rules expect this wrapper).
            let body_start = self.start();
            let body_id = self.reserve(parent);
            let expr_start = self.start();
            let expr = self.parse_assignment_expression(Some(body_id));
            let expr_end = self.tree.span(expr).map_or(self.prev_end, |s| s.end);
            let es_id = self.push(
                AstNode::ExpressionStatement(starlint_ast::node::ExpressionStatementNode {
                    span: Span::new(expr_start, expr_end),
                    expression: expr,
                }),
                Some(body_id),
            );
            self.tree.set(
                body_id,
                AstNode::FunctionBody(starlint_ast::node::FunctionBodyNode {
                    span: Span::new(body_start, expr_end),
                    statements: Box::new([es_id]),
                }),
            );
            body_id
        }
    }

    /// Parse a template literal.
    pub(crate) fn parse_template_literal(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let template_id = self.reserve(parent);
        let mut expressions = Vec::new();
        let mut quasis = Vec::new();

        match self.cur() {
            TokenKind::NoSubstitutionTemplate => {
                // Extract raw text between backticks: `text`
                let raw = self.extract_template_raw(self.current.start, self.current.end);
                quasis.push(raw);
                let tok = self.bump();
                self.tree.set(
                    template_id,
                    AstNode::TemplateLiteral(TemplateLiteralNode {
                        span: Span::new(start, tok.end),
                        quasis: quasis.into_boxed_slice(),
                        expressions: Box::new([]),
                    }),
                );
            }
            TokenKind::TemplateHead => {
                // Extract raw text of head: `text${
                let raw = self.extract_template_raw(self.current.start, self.current.end);
                quasis.push(raw);
                self.bump();
                loop {
                    // Parse expression inside `${ ... }`
                    let expr = self.parse_expression(Some(template_id));
                    expressions.push(expr);

                    match self.cur() {
                        TokenKind::TemplateTail => {
                            // Extract raw text of tail: }text`
                            let raw =
                                self.extract_template_raw(self.current.start, self.current.end);
                            quasis.push(raw);
                            let tok = self.bump();
                            self.tree.set(
                                template_id,
                                AstNode::TemplateLiteral(TemplateLiteralNode {
                                    span: Span::new(start, tok.end),
                                    quasis: quasis.into_boxed_slice(),
                                    expressions: expressions.into_boxed_slice(),
                                }),
                            );
                            break;
                        }
                        TokenKind::TemplateMiddle => {
                            // Extract raw text of middle: }text${
                            let raw =
                                self.extract_template_raw(self.current.start, self.current.end);
                            quasis.push(raw);
                            self.bump();
                            // Continue to next expression
                        }
                        _ => {
                            // Error recovery
                            self.error("expected template continuation");
                            let end = self.prev_end;
                            self.tree.set(
                                template_id,
                                AstNode::TemplateLiteral(TemplateLiteralNode {
                                    span: Span::new(start, end),
                                    quasis: quasis.into_boxed_slice(),
                                    expressions: expressions.into_boxed_slice(),
                                }),
                            );
                            break;
                        }
                    }
                }
            }
            _ => {
                self.error("expected template literal");
                self.tree.set(
                    template_id,
                    AstNode::TemplateLiteral(TemplateLiteralNode {
                        span: Span::new(start, self.prev_end),
                        quasis: Box::new([]),
                        expressions: Box::new([]),
                    }),
                );
            }
        }

        template_id
    }

    /// Extract raw text from a template token, stripping delimiters
    /// (backtick, `}`, `${`).
    fn extract_template_raw(&self, tok_start: u32, tok_end: u32) -> String {
        let text = self.text(tok_start, tok_end);
        let bytes = text.as_bytes();
        let len = bytes.len();
        if len < 2 {
            return String::new();
        }
        // Skip leading delimiter (` or })
        let s = usize::from(bytes[0] == b'`' || bytes[0] == b'}');
        // Skip trailing delimiter (` or ${)
        let e = if bytes.last().copied() == Some(b'`') {
            len.saturating_sub(1)
        } else if bytes.last().copied() == Some(b'{')
            && len >= 3
            && bytes.get(len.saturating_sub(2)).copied() == Some(b'$')
        {
            len.saturating_sub(2)
        } else {
            len
        };
        if s >= e {
            return String::new();
        }
        text.get(s..e).unwrap_or_default().to_owned()
    }

    /// Parse an array literal `[a, b, c]`.
    fn parse_array_literal(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let arr_id = self.reserve(parent);
        self.bump(); // `[`
        let mut elements = Vec::new();

        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Comma) {
                // Elision (hole)
                self.bump();
                continue;
            }
            if self.at(TokenKind::DotDotDot) {
                let spread_start = self.start();
                let spread_id = self.reserve(Some(arr_id));
                self.bump();
                let arg = self.parse_assignment_expression(Some(spread_id));
                let end = self.tree.span(arg).map_or(0, |s| s.end);
                self.tree.set(
                    spread_id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Span::new(spread_start, end),
                        argument: arg,
                    }),
                );
                elements.push(spread_id);
            } else {
                let elem = self.parse_assignment_expression(Some(arr_id));
                elements.push(elem);
            }
            if !self.at(TokenKind::RBracket) {
                let _ = self.expect(TokenKind::Comma);
            }
        }

        let end_tok = self.current.end;
        let _ = self.expect(TokenKind::RBracket);
        self.tree.set(
            arr_id,
            AstNode::ArrayExpression(ArrayExpressionNode {
                span: Span::new(start, end_tok),
                elements: elements.into_boxed_slice(),
            }),
        );
        arr_id
    }

    /// Parse an object literal `{ a: 1, b: 2 }`.
    fn parse_object_literal(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let obj_id = self.reserve(parent);
        self.bump(); // `{`
        let mut properties = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::DotDotDot) {
                let spread_start = self.start();
                let spread_id = self.reserve(Some(obj_id));
                self.bump();
                let arg = self.parse_assignment_expression(Some(spread_id));
                let end = self.tree.span(arg).map_or(0, |s| s.end);
                self.tree.set(
                    spread_id,
                    AstNode::SpreadElement(SpreadElementNode {
                        span: Span::new(spread_start, end),
                        argument: arg,
                    }),
                );
                properties.push(spread_id);
            } else {
                let prop = self.parse_object_property(Some(obj_id));
                properties.push(prop);
            }
            if !self.at(TokenKind::RBrace) {
                // Allow trailing comma
                self.eat(TokenKind::Comma);
            }
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);
        self.tree.set(
            obj_id,
            AstNode::ObjectExpression(ObjectExpressionNode {
                span: Span::new(start, end),
                properties: properties.into_boxed_slice(),
            }),
        );
        obj_id
    }

    /// Parse a single object property.
    fn parse_object_property(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let prop_id = self.reserve(parent);

        // Detect getter/setter: `get name() {}` or `set name(v) {}`
        let kind = if self.at(TokenKind::Get) || self.at(TokenKind::Set) {
            let is_get = self.at(TokenKind::Get);
            let next = self.peek_next_text();
            // Not a getter/setter if followed by `(`, `:`, `,`, `}`, `=`, `;`, or EOF.
            // Those indicate: method named get/set, regular prop, shorthand, etc.
            if matches!(next, "(" | ":" | "," | "}" | "=" | ";" | "") {
                PropertyKind::Init
            } else {
                self.bump(); // consume `get`/`set`
                if is_get {
                    PropertyKind::Get
                } else {
                    PropertyKind::Set
                }
            }
        } else {
            PropertyKind::Init
        };

        // Computed key: `[expr]: value`
        let (key, computed, shorthand) = if self.at(TokenKind::LBracket) {
            self.bump(); // `[`
            let k = self.parse_assignment_expression(Some(prop_id));
            let _ = self.expect(TokenKind::RBracket);
            (k, true, false)
        } else {
            // Identifier, string, or number key
            let key_start = self.start();
            let key_text = self.cur_text().to_owned();
            let key_tok = self.bump();
            let k = self.push(
                AstNode::IdentifierReference(IdentifierReferenceNode {
                    span: Span::new(key_start, key_tok.end),
                    name: key_text,
                }),
                Some(prop_id),
            );

            // Check for shorthand: `{ x }` (no colon, not getter/setter)
            if kind == PropertyKind::Init
                && !self.at(TokenKind::Colon)
                && !self.at(TokenKind::LParen)
                && (key_tok.kind == TokenKind::Identifier || key_tok.kind.is_keyword())
            {
                // Shorthand property: key is also value
                let val = self.push(
                    AstNode::IdentifierReference(IdentifierReferenceNode {
                        span: Span::new(key_start, key_tok.end),
                        name: self.text(key_start, key_tok.end).to_owned(),
                    }),
                    Some(prop_id),
                );
                self.tree.set(
                    prop_id,
                    AstNode::ObjectProperty(ObjectPropertyNode {
                        span: Span::new(start, key_tok.end),
                        key: k,
                        value: val,
                        kind,
                        computed: false,
                        shorthand: true,
                        method: false,
                    }),
                );
                return prop_id;
            }

            (k, false, false)
        };

        // Method shorthand: `{ method() { ... } }` or getter/setter
        if self.at(TokenKind::LParen) {
            let func = self.parse_function_expression_body(Some(prop_id), false, false);
            let end = self.tree.span(func).map_or(0, |s| s.end);
            self.tree.set(
                prop_id,
                AstNode::ObjectProperty(ObjectPropertyNode {
                    span: Span::new(start, end),
                    key,
                    value: func,
                    kind,
                    computed,
                    shorthand: false,
                    method: kind == PropertyKind::Init,
                }),
            );
            return prop_id;
        }

        // Regular property: `key: value`
        let _ = self.expect(TokenKind::Colon);
        let value = self.parse_assignment_expression(Some(prop_id));
        let end = self.tree.span(value).map_or(0, |s| s.end);
        self.tree.set(
            prop_id,
            AstNode::ObjectProperty(ObjectPropertyNode {
                span: Span::new(start, end),
                key,
                value,
                kind,
                computed,
                shorthand,
                method: false,
            }),
        );
        prop_id
    }
}

// --- Utility functions ---

/// Parse a numeric string to f64.
fn parse_number(text: &str) -> f64 {
    if text.starts_with("0x") || text.starts_with("0X") {
        #[allow(clippy::as_conversions)]
        let without_prefix = text.get(2..).unwrap_or_default().replace('_', "");
        return u64::from_str_radix(&without_prefix, 16).map_or(f64::NAN, |v| v as f64);
    }
    if text.starts_with("0o") || text.starts_with("0O") {
        #[allow(clippy::as_conversions)]
        let without_prefix = text.get(2..).unwrap_or_default().replace('_', "");
        return u64::from_str_radix(&without_prefix, 8).map_or(f64::NAN, |v| v as f64);
    }
    if text.starts_with("0b") || text.starts_with("0B") {
        #[allow(clippy::as_conversions)]
        let without_prefix = text.get(2..).unwrap_or_default().replace('_', "");
        return u64::from_str_radix(&without_prefix, 2).map_or(f64::NAN, |v| v as f64);
    }
    // Strip BigInt suffix and separators
    let cleaned = text.trim_end_matches('n').replace('_', "");
    cleaned.parse::<f64>().unwrap_or(f64::NAN)
}

/// Parse a regex literal `/pattern/flags`.
fn parse_regex(raw: &str) -> (String, String) {
    // Find the last `/` that terminates the pattern
    if let Some(last_slash) = raw.rfind('/') {
        if last_slash > 0 {
            let pattern = raw.get(1..last_slash).unwrap_or_default().to_owned();
            let flags = raw
                .get(last_slash.saturating_add(1)..)
                .unwrap_or_default()
                .to_owned();
            return (pattern, flags);
        }
    }
    (raw.to_owned(), String::new())
}
