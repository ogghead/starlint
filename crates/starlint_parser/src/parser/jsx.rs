//! JSX parsing.

use starlint_ast::node::{
    AstNode, JSXAttributeNode, JSXElementNode, JSXExpressionContainerNode, JSXFragmentNode,
    JSXOpeningElementNode, JSXSpreadAttributeNode, JSXTextNode, StringLiteralNode,
};
use starlint_ast::types::{NodeId, Span};

use crate::lexer::LexerMode;
use crate::token::TokenKind;

use super::Parser;

impl Parser<'_> {
    /// Parse a JSX element (opening tag, children, closing tag) or fragment.
    pub(crate) fn parse_jsx_element(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `<`

        // Fragment: `<>...</>`
        if self.at(TokenKind::RAngle) {
            return self.parse_jsx_fragment(parent, start);
        }

        // Closing tag without opening (error recovery or fragment closing)
        if self.at(TokenKind::Slash) {
            // This might be `</` — handled by caller
            self.error("unexpected JSX closing tag");
            self.bump();
            return self.push(
                AstNode::Unknown(starlint_ast::node::UnknownNode {
                    span: Span::new(start, self.prev_end),
                }),
                parent,
            );
        }

        let element_id = self.reserve(parent);

        // Parse opening element
        let opening_id = self.reserve(Some(element_id));
        self.lexer.set_mode(LexerMode::JsxTag);

        // Tag name
        let tag_name = self.parse_jsx_tag_name();
        // Attributes
        let mut attributes = Vec::new();
        while !self.at(TokenKind::RAngle) && !self.at(TokenKind::Slash) && !self.at(TokenKind::Eof)
        {
            let attr = self.parse_jsx_attribute(Some(opening_id));
            attributes.push(attr);
        }

        // Self-closing `/>` or `>`
        let is_self_closing = if self.at(TokenKind::Slash) {
            self.bump(); // `/`
            true
        } else {
            false
        };
        if self.at(TokenKind::RAngle) {
            // Switch mode BEFORE bumping so the next token is lexed correctly.
            if is_self_closing {
                self.lexer.set_mode(LexerMode::Normal);
            } else {
                self.lexer.set_mode(LexerMode::JsxChild);
            }
            self.bump(); // `>`
        }

        let opening_end = self.prev_end;
        self.tree.set(
            opening_id,
            AstNode::JSXOpeningElement(JSXOpeningElementNode {
                span: Span::new(start, opening_end),
                name: tag_name,
                attributes: attributes.into_boxed_slice(),
                self_closing: is_self_closing,
            }),
        );

        if is_self_closing {
            self.tree.set(
                element_id,
                AstNode::JSXElement(JSXElementNode {
                    span: Span::new(start, opening_end),
                    opening_element: opening_id,
                    children: Box::new([]),
                }),
            );
            return element_id;
        }

        // Children
        self.lexer.set_mode(LexerMode::JsxChild);
        let children = self.parse_jsx_children(Some(element_id));
        self.lexer.set_mode(LexerMode::Normal);

        // Closing tag `</TagName>`
        // We should be at `<` from the child scan
        if self.at(TokenKind::LAngle) {
            self.bump(); // `<`
        }
        if self.at(TokenKind::Slash) {
            self.bump(); // `/`
        }
        // Skip closing tag name
        self.lexer.set_mode(LexerMode::JsxTag);
        while !self.at(TokenKind::RAngle) && !self.at(TokenKind::Eof) {
            self.bump();
        }
        let end = self.current.end;
        if self.at(TokenKind::RAngle) {
            self.bump();
        }
        self.lexer.set_mode(LexerMode::Normal);

        self.tree.set(
            element_id,
            AstNode::JSXElement(JSXElementNode {
                span: Span::new(start, end),
                opening_element: opening_id,
                children: children.into_boxed_slice(),
            }),
        );
        element_id
    }

    /// Parse a JSX fragment `<>...</>`.
    fn parse_jsx_fragment(&mut self, parent: Option<NodeId>, start: u32) -> NodeId {
        let frag_id = self.reserve(parent);
        self.bump(); // `>` (opening)

        self.lexer.set_mode(LexerMode::JsxChild);
        let children = self.parse_jsx_children(Some(frag_id));
        self.lexer.set_mode(LexerMode::Normal);

        // `</>`
        if self.at(TokenKind::LAngle) {
            self.bump();
        }
        if self.at(TokenKind::Slash) {
            self.bump();
        }
        let end = self.current.end;
        if self.at(TokenKind::RAngle) {
            self.bump();
        }

        self.tree.set(
            frag_id,
            AstNode::JSXFragment(JSXFragmentNode {
                span: Span::new(start, end),
                children: children.into_boxed_slice(),
            }),
        );
        frag_id
    }

    /// Parse JSX children (text, expressions, nested elements).
    fn parse_jsx_children(&mut self, parent: Option<NodeId>) -> Vec<NodeId> {
        let mut children = Vec::new();

        loop {
            match self.cur() {
                TokenKind::Eof => break,
                TokenKind::JsxText => {
                    let text_start = self.current.start;
                    let text_value = self.cur_text().to_owned();
                    let tok = self.bump();
                    let child = self.push(
                        AstNode::JSXText(JSXTextNode {
                            span: Span::new(text_start, tok.end),
                            value: text_value,
                        }),
                        parent,
                    );
                    children.push(child);
                }
                TokenKind::LBrace => {
                    let expr_start = self.current.start;
                    self.lexer.set_mode(LexerMode::Normal);
                    self.bump(); // `{` — next token lexed in Normal mode
                    let container_id = self.reserve(parent);
                    let expression = if self.at(TokenKind::RBrace) {
                        None
                    } else {
                        Some(self.parse_expression(Some(container_id)))
                    };
                    let end = self.current.end;
                    let _ = self.expect(TokenKind::RBrace);
                    self.lexer.set_mode(LexerMode::JsxChild);
                    self.tree.set(
                        container_id,
                        AstNode::JSXExpressionContainer(JSXExpressionContainerNode {
                            span: Span::new(expr_start, end),
                            expression,
                        }),
                    );
                    children.push(container_id);
                }
                TokenKind::LAngle => {
                    // Could be nested element or closing tag `</`
                    // Check for closing tag
                    #[allow(clippy::as_conversions)]
                    let next = self
                        .source
                        .get(self.current.end as usize..)
                        .unwrap_or_default();
                    if next.starts_with('/') {
                        // Closing tag — end of children
                        break;
                    }
                    // Nested element
                    self.lexer.set_mode(LexerMode::Normal);
                    let child = self.parse_jsx_element(parent);
                    self.lexer.set_mode(LexerMode::JsxChild);
                    children.push(child);
                }
                _ => break,
            }
        }

        children
    }

    /// Parse a JSX tag name (identifier, member expression, or namespaced).
    fn parse_jsx_tag_name(&mut self) -> String {
        let mut name = self.cur_text().to_owned();
        self.bump();

        // Member expression: `Foo.Bar.Baz`
        while self.at(TokenKind::Dot) {
            name.push('.');
            self.bump();
            name.push_str(self.cur_text());
            self.bump();
        }

        // Namespaced name: `ns:name`
        if self.at(TokenKind::Colon) {
            name.push(':');
            self.bump();
            name.push_str(self.cur_text());
            self.bump();
        }

        name
    }

    /// Parse a JSX attribute.
    fn parse_jsx_attribute(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();

        // Spread attribute: `{...expr}`
        if self.at(TokenKind::LBrace) {
            let spread_id = self.reserve(parent);
            // Switch to Normal mode BEFORE bumping so `...` is lexed correctly.
            self.lexer.set_mode(LexerMode::Normal);
            self.bump(); // `{`
            let _ = self.expect(TokenKind::DotDotDot);
            let argument = self.parse_assignment_expression(Some(spread_id));
            let end = self.current.end;
            let _ = self.expect(TokenKind::RBrace);
            self.lexer.set_mode(LexerMode::JsxTag);
            self.tree.set(
                spread_id,
                AstNode::JSXSpreadAttribute(JSXSpreadAttributeNode {
                    span: Span::new(start, end),
                    argument,
                }),
            );
            return spread_id;
        }

        let attr_id = self.reserve(parent);
        let mut attr_name = self.cur_text().to_owned();
        self.bump();

        // Namespaced attribute: `ns:name` → attr_name = "ns:name"
        if self.at(TokenKind::Colon) {
            self.bump();
            attr_name.push(':');
            attr_name.push_str(self.cur_text());
            self.bump();
        }

        // Value
        let value = if self.eat(TokenKind::Eq) {
            if self.at(TokenKind::String) {
                let val_start = self.start();
                let raw = self.cur_text();
                let val = if raw.len() >= 2 {
                    raw.get(1..raw.len().saturating_sub(1))
                        .unwrap_or_default()
                        .to_owned()
                } else {
                    String::new()
                };
                let tok = self.bump();
                Some(self.push(
                    AstNode::StringLiteral(StringLiteralNode {
                        span: Span::new(val_start, tok.end),
                        value: val,
                    }),
                    Some(attr_id),
                ))
            } else if self.at(TokenKind::LBrace) {
                let expr_start = self.start();
                self.lexer.set_mode(LexerMode::Normal);
                self.bump(); // `{` — next token lexed in Normal mode
                let container_id = self.reserve(Some(attr_id));
                let expression = Some(self.parse_assignment_expression(Some(container_id)));
                let end = self.current.end;
                let _ = self.expect(TokenKind::RBrace);
                self.lexer.set_mode(LexerMode::JsxTag);
                self.tree.set(
                    container_id,
                    AstNode::JSXExpressionContainer(JSXExpressionContainerNode {
                        span: Span::new(expr_start, end),
                        expression,
                    }),
                );
                Some(container_id)
            } else {
                None
            }
        } else {
            None
        };

        let end = value
            .and_then(|id| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end);

        self.tree.set(
            attr_id,
            AstNode::JSXAttribute(JSXAttributeNode {
                span: Span::new(start, end),
                name: attr_name,
                value,
            }),
        );
        attr_id
    }
}
