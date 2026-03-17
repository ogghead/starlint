//! Postfix operations: postfix update, call expressions, member access,
//! optional chaining, tagged templates, new expressions, and argument parsing.

use starlint_ast::node::{
    AstNode, CallExpressionNode, ChainExpressionNode, ComputedMemberExpressionNode,
    NewExpressionNode, SpreadElementNode, StaticMemberExpressionNode, TSNonNullExpressionNode,
    TaggedTemplateExpressionNode, UpdateExpressionNode,
};
use starlint_ast::operator::UpdateOperator;
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    /// Parse a postfix update expression (`x++`, `x--`).
    pub(in crate::parser) fn parse_update_expression(&mut self, parent: Option<NodeId>) -> NodeId {
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
}
