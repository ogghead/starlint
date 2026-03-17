//! Array literals, object literals, template literals, regex literals,
//! and numeric/string utility functions.

use starlint_ast::node::{
    ArrayExpressionNode, AstNode, IdentifierReferenceNode, ObjectExpressionNode,
    ObjectPropertyNode, SpreadElementNode, TemplateLiteralNode,
};
use starlint_ast::operator::PropertyKind;
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

// --- Utility functions ---

/// Strip numeric separators (`_`) only when present, avoiding allocation for
/// the common case where no separators exist.
fn strip_separators(s: &str) -> std::borrow::Cow<'_, str> {
    if s.contains('_') {
        std::borrow::Cow::Owned(s.replace('_', ""))
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}

/// Parse a numeric string to f64.
pub(in crate::parser::expressions) fn parse_number(text: &str) -> f64 {
    if text.starts_with("0x") || text.starts_with("0X") {
        #[allow(clippy::as_conversions)]
        let without_prefix = strip_separators(text.get(2..).unwrap_or_default());
        return u64::from_str_radix(&without_prefix, 16).map_or(f64::NAN, |v| v as f64);
    }
    if text.starts_with("0o") || text.starts_with("0O") {
        #[allow(clippy::as_conversions)]
        let without_prefix = strip_separators(text.get(2..).unwrap_or_default());
        return u64::from_str_radix(&without_prefix, 8).map_or(f64::NAN, |v| v as f64);
    }
    if text.starts_with("0b") || text.starts_with("0B") {
        #[allow(clippy::as_conversions)]
        let without_prefix = strip_separators(text.get(2..).unwrap_or_default());
        return u64::from_str_radix(&without_prefix, 2).map_or(f64::NAN, |v| v as f64);
    }
    // Strip BigInt suffix and separators.
    let trimmed = text.trim_end_matches('n');
    let cleaned = strip_separators(trimmed);
    cleaned.parse::<f64>().unwrap_or(f64::NAN)
}

/// Process JavaScript escape sequences in a string literal.
pub(in crate::parser::expressions) fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('v') => result.push('\u{000B}'),
                Some(other) => {
                    // Unknown escapes: keep as-is (e.g. \u, \x — leave raw)
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse a regex literal `/pattern/flags`, returning slices into the raw text.
pub(in crate::parser::expressions) fn parse_regex(raw: &str) -> (&str, &str) {
    // Find the last `/` that terminates the pattern
    if let Some(last_slash) = raw.rfind('/') {
        if last_slash > 0 {
            let pattern = raw.get(1..last_slash).unwrap_or_default();
            let flags = raw.get(last_slash.saturating_add(1)..).unwrap_or_default();
            return (pattern, flags);
        }
    }
    (raw, "")
}

impl Parser<'_> {
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
    pub(in crate::parser) fn parse_array_literal(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(in crate::parser) fn parse_object_literal(&mut self, parent: Option<NodeId>) -> NodeId {
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
