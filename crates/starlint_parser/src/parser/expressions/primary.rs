//! Primary/atom expressions: identifiers, simple literals, parenthesized
//! expressions, super, and arrow function parsing.

use starlint_ast::node::{
    ArrowFunctionExpressionNode, AssignmentPatternNode, AstNode, BindingIdentifierNode,
    BooleanLiteralNode, IdentifierReferenceNode, NullLiteralNode, NumericLiteralNode,
    RegExpLiteralNode, StringLiteralNode, ThisExpressionNode, UnknownNode,
};
use starlint_ast::operator::AssignmentOperator;
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;
use super::literal::{parse_number, parse_regex, unescape_string};

impl Parser<'_> {
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
                // Strip quotes and process escape sequences
                let value = if raw.len() >= 2 {
                    let inner = raw.get(1..raw.len().saturating_sub(1)).unwrap_or_default();
                    if inner.contains('\\') {
                        unescape_string(inner)
                    } else {
                        inner.to_owned()
                    }
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
                        pattern: pattern.to_owned(),
                        flags: flags.to_owned(),
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
            TokenKind::LAngle if self.options.typescript && !self.options.jsx => {
                // TypeScript angle-bracket type assertion: `<Type>expr`
                self.parse_ts_type_assertion(parent)
            }
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

        // Apply cover grammar: convert expression nodes to binding/pattern nodes.
        for &param_id in params {
            self.convert_expr_to_param(param_id);
        }

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

    /// Convert expression nodes to binding/pattern nodes for arrow function cover grammar.
    ///
    /// When `(a, b = 1)` is parsed as expressions and then `=>` is found, the
    /// expression nodes must be reinterpreted: `IdentifierReference` becomes
    /// `BindingIdentifier`, `AssignmentExpression(=)` becomes `AssignmentPattern`.
    fn convert_expr_to_param(&mut self, id: NodeId) {
        let node = self.tree.get(id).cloned();
        match node {
            Some(AstNode::IdentifierReference(ident)) => {
                self.tree.set(
                    id,
                    AstNode::BindingIdentifier(BindingIdentifierNode {
                        span: ident.span,
                        name: ident.name,
                    }),
                );
            }
            Some(AstNode::AssignmentExpression(assign))
                if assign.operator == AssignmentOperator::Assign =>
            {
                // Convert `left = right` to `AssignmentPattern { left, right }`
                // First convert the left side recursively
                self.convert_expr_to_param(assign.left);
                self.tree.set(
                    id,
                    AstNode::AssignmentPattern(AssignmentPatternNode {
                        span: assign.span,
                        left: assign.left,
                        right: assign.right,
                    }),
                );
            }
            _ => {
                // Other expression types (ObjectExpression, ArrayExpression, etc.)
                // are left as-is for now — rules handle them adequately.
            }
        }
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
}
