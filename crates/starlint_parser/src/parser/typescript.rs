//! TypeScript-specific parsing (type annotations, declarations).

use starlint_ast::node::{
    AstNode, TSAnyKeywordNode, TSAsExpressionNode, TSEnumDeclarationNode, TSEnumMemberNode,
    TSInterfaceDeclarationNode, TSModuleDeclarationNode, TSTypeAliasDeclarationNode,
    TSTypeAssertionNode, TSTypeLiteralNode, TSTypeParameterNode, TSTypeReferenceNode,
    TSVoidKeywordNode, UnknownNode,
};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::Parser;

impl Parser<'_> {
    /// Parse a TypeScript type annotation (after `:`).
    ///
    /// This is a simplified type parser that handles the most common type
    /// constructs needed by lint rules. Complex types are mapped to `Unknown`.
    pub(crate) fn parse_ts_type(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();

        let base_type = match self.cur() {
            // Keyword types
            TokenKind::Any => {
                let tok = self.bump();
                self.push(
                    AstNode::TSAnyKeyword(TSAnyKeywordNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
            TokenKind::Void => {
                let tok = self.bump();
                self.push(
                    AstNode::TSVoidKeyword(TSVoidKeywordNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
            TokenKind::Unknown
            | TokenKind::Never
            | TokenKind::Null
            | TokenKind::True
            | TokenKind::False => {
                let tok = self.bump();
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
            TokenKind::String | TokenKind::Number => {
                // String/number literal types
                let tok = self.bump();
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
            // Object type literal: `{ ... }`
            TokenKind::LBrace => self.parse_ts_type_literal(parent),
            // Tuple type: `[T, U]`
            TokenKind::LBracket => self.parse_ts_tuple_type(parent),
            // Function type: `(params) => ReturnType`
            TokenKind::LParen => self.parse_ts_function_type(parent),
            // `typeof x`
            TokenKind::Typeof => {
                self.bump();
                let _ref_type = self.parse_ts_type(parent);
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, self.prev_end),
                    }),
                    parent,
                )
            }
            // `keyof T`
            TokenKind::Keyof => {
                self.bump();
                let _inner = self.parse_ts_type(parent);
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, self.prev_end),
                    }),
                    parent,
                )
            }
            // `readonly T[]`
            TokenKind::Readonly => {
                self.bump();
                self.parse_ts_type(parent)
            }
            // `infer T`
            TokenKind::Infer => {
                self.bump();
                let _ = self.parse_binding_identifier(parent);
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, self.prev_end),
                    }),
                    parent,
                )
            }
            // Type reference: `Foo`, `Array<T>`, `ns.Type`
            TokenKind::Identifier
            | TokenKind::Interface
            | TokenKind::Type
            | TokenKind::From
            | TokenKind::Of
            | TokenKind::Get
            | TokenKind::Set
            | TokenKind::Async
            | TokenKind::Is
            | TokenKind::Asserts
            | TokenKind::Namespace
            | TokenKind::Module
            | TokenKind::Declare
            | TokenKind::Abstract
            | TokenKind::Override
            | TokenKind::Satisfies
            | TokenKind::Using => self.parse_ts_type_reference(parent),
            // Conditional negative: `T extends U ? X : Y` handled in union parsing
            _ => {
                self.error(format!("unexpected token in type: {:?}", self.cur()));
                let tok = self.bump();
                self.push(
                    AstNode::Unknown(UnknownNode {
                        span: Span::new(start, tok.end),
                    }),
                    parent,
                )
            }
        };

        // Handle postfix type operators
        self.parse_ts_type_postfix(base_type, parent)
    }

    /// Parse postfix type operators (`[]`, `extends`, `|`, `&`).
    fn parse_ts_type_postfix(&mut self, base: NodeId, parent: Option<NodeId>) -> NodeId {
        let result = base;

        loop {
            match self.cur() {
                // Array type: `T[]`
                TokenKind::LBracket if self.peek_is_rbracket() => {
                    self.bump(); // `[`
                    self.bump(); // `]`
                    // Keep result, just extend span
                }
                // Union type: `T | U`
                TokenKind::Pipe => {
                    self.bump();
                    let _right = self.parse_ts_type(parent);
                    // Simplified: don't create union node, just parse through
                }
                // Intersection type: `T & U`
                TokenKind::Amp => {
                    self.bump();
                    let _right = self.parse_ts_type(parent);
                }
                // Conditional type: `T extends U ? X : Y`
                TokenKind::Extends => {
                    self.bump();
                    let _check = self.parse_ts_type(parent);
                    if self.eat(TokenKind::Question) {
                        let _true_type = self.parse_ts_type(parent);
                        let _ = self.expect(TokenKind::Colon);
                        let _false_type = self.parse_ts_type(parent);
                    }
                }
                _ => break,
            }
        }

        result
    }

    /// Check if `[` is immediately followed by `]` (array type notation).
    fn peek_is_rbracket(&self) -> bool {
        #[allow(clippy::as_conversions)]
        let after = self
            .source
            .get(self.current.end as usize..)
            .unwrap_or_default()
            .trim_start();
        after.starts_with(']')
    }

    /// Parse a type reference (`Foo`, `Array<T>`, `ns.Type`).
    fn parse_ts_type_reference(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let ref_id = self.reserve(parent);

        // Type name (may be dotted: `ns.Type`)
        let mut name = self.cur_text().to_owned();
        self.bump();
        while self.at(TokenKind::Dot) {
            name.push('.');
            self.bump();
            name.push_str(self.cur_text());
            self.bump();
        }

        // Type arguments: `<T, U>`
        let type_arguments = if self.at(TokenKind::LAngle) {
            self.parse_ts_type_arguments(Some(ref_id))
        } else {
            Vec::new()
        };

        let end = self.prev_end;
        self.tree.set(
            ref_id,
            AstNode::TSTypeReference(TSTypeReferenceNode {
                span: Span::new(start, end),
                type_name: name,
                type_arguments: type_arguments.into_boxed_slice(),
            }),
        );
        ref_id
    }

    /// Parse type arguments `<T, U>`.
    fn parse_ts_type_arguments(&mut self, parent: Option<NodeId>) -> Vec<NodeId> {
        self.bump(); // `<`
        let mut args = Vec::new();
        while !self.at(TokenKind::RAngle) && !self.at(TokenKind::Eof) {
            let arg = self.parse_ts_type(parent);
            args.push(arg);
            if !self.at(TokenKind::RAngle) {
                self.eat(TokenKind::Comma);
            }
        }
        if self.at(TokenKind::RAngle) {
            self.bump();
        }
        args
    }

    /// Parse type parameters `<T, U extends V>`.
    pub(crate) fn parse_ts_type_parameters(&mut self, parent: Option<NodeId>) -> Vec<NodeId> {
        self.bump(); // `<`
        let mut params = Vec::new();
        while !self.at(TokenKind::RAngle) && !self.at(TokenKind::Eof) {
            let param = self.parse_ts_type_parameter(parent);
            params.push(param);
            if !self.at(TokenKind::RAngle) {
                self.eat(TokenKind::Comma);
            }
        }
        if self.at(TokenKind::RAngle) {
            self.bump();
        }
        params
    }

    /// Parse a single type parameter `T extends U = Default`.
    fn parse_ts_type_parameter(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let param_id = self.reserve(parent);
        let name = self.cur_text().to_owned();
        self.bump();

        let constraint = self
            .eat(TokenKind::Extends)
            .then(|| self.parse_ts_type(Some(param_id)));

        let default = self
            .eat(TokenKind::Eq)
            .then(|| self.parse_ts_type(Some(param_id)));

        let end = self.prev_end;
        self.tree.set(
            param_id,
            AstNode::TSTypeParameter(TSTypeParameterNode {
                span: Span::new(start, end),
                name,
                constraint,
                default,
            }),
        );
        param_id
    }

    /// Skip type parameters and/or parenthesized parameter lists.
    fn skip_paren_or_type_params(&mut self) {
        if self.at(TokenKind::LAngle) {
            self.skip_ts_type_parameters();
        }
        if self.at(TokenKind::LParen) {
            self.bump(); // `(`
            let mut depth = 1u32;
            while depth > 0 && !self.at(TokenKind::Eof) {
                match self.cur() {
                    TokenKind::LParen => {
                        depth = depth.saturating_add(1);
                        self.bump();
                    }
                    TokenKind::RParen => {
                        depth = depth.saturating_sub(1);
                        self.bump();
                    }
                    _ => {
                        self.bump();
                    }
                }
            }
        }
    }

    /// Skip type parameters `<...>` without creating AST nodes.
    fn skip_ts_type_parameters(&mut self) {
        if !self.at(TokenKind::LAngle) {
            return;
        }
        self.bump(); // `<`
        let mut depth = 1u32;
        while depth > 0 && !self.at(TokenKind::Eof) {
            match self.cur() {
                TokenKind::LAngle => {
                    depth = depth.saturating_add(1);
                    self.bump();
                }
                TokenKind::RAngle => {
                    depth = depth.saturating_sub(1);
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// Parse a type literal `{ ... }`.
    fn parse_ts_type_literal(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let lit_id = self.reserve(parent);
        self.bump(); // `{`

        let mut members = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            // Simplified: skip member parsing, just balance braces
            let member = self.parse_ts_type_member(Some(lit_id));
            members.push(member);
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            lit_id,
            AstNode::TSTypeLiteral(TSTypeLiteralNode {
                span: Span::new(start, end),
                members: members.into_boxed_slice(),
            }),
        );
        lit_id
    }

    /// Parse a type member (property signature, method signature, call signature).
    ///
    /// Handles `name: Type`, `name?: Type`, `[key: Type]: Type`,
    /// method signatures, and call signatures `(): Type`.
    /// The member node itself is `Unknown` but the type annotation is properly
    /// parsed so downstream rules can see type references.
    fn parse_ts_type_member(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let member_id = self.reserve(parent);

        // Skip `readonly` modifier
        if self.at(TokenKind::Identifier) && self.cur_text() == "readonly" {
            self.bump();
        }

        // Call/construct signature: `(): Type`, `<T>(params): Type`, `new (): Type`
        if self.at(TokenKind::LParen) || self.at(TokenKind::LAngle) {
            self.skip_paren_or_type_params();
        } else {
            // Computed property name: `[key: Type]`
            if self.at(TokenKind::LBracket) {
                self.bump(); // `[`
                let mut depth = 1u32;
                while depth > 0 && !self.at(TokenKind::Eof) {
                    match self.cur() {
                        TokenKind::LBracket => {
                            depth = depth.saturating_add(1);
                            self.bump();
                        }
                        TokenKind::RBracket => {
                            depth = depth.saturating_sub(1);
                            self.bump();
                        }
                        _ => {
                            self.bump();
                        }
                    }
                }
            } else {
                // Property name (identifier, string, or keyword used as name)
                self.bump();
            }

            // Optional `?`
            self.eat(TokenKind::Question);

            // Method signature: `name(...)` or `name<T>(...)`
            if self.at(TokenKind::LParen) || self.at(TokenKind::LAngle) {
                self.skip_paren_or_type_params();
            }
        }

        // Type annotation: `: Type`
        if self.at(TokenKind::Colon) {
            self.bump(); // `:`
            let _type_id = self.parse_ts_type(Some(member_id));
        }

        // Consume separator (`;` or `,`)
        if self.at(TokenKind::Semicolon) || self.at(TokenKind::Comma) {
            self.bump();
        }

        self.tree.set(
            member_id,
            AstNode::Unknown(UnknownNode {
                span: Span::new(start, self.prev_end),
            }),
        );
        member_id
    }

    /// Parse a tuple type `[T, U, V]`.
    fn parse_ts_tuple_type(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `[`
        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            let _elem = self.parse_ts_type(parent);
            self.eat(TokenKind::Comma);
        }
        let end = self.current.end;
        let _ = self.expect(TokenKind::RBracket);
        self.push(
            AstNode::Unknown(UnknownNode {
                span: Span::new(start, end),
            }),
            parent,
        )
    }

    /// Parse a function type `(params) => ReturnType`.
    fn parse_ts_function_type(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `(`
        // Skip params
        let mut depth = 1u32;
        while depth > 0 {
            match self.cur() {
                TokenKind::Eof => break,
                TokenKind::LParen => {
                    depth = depth.saturating_add(1);
                    self.bump();
                }
                TokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
        if self.at(TokenKind::Arrow) {
            self.bump();
            let _ret = self.parse_ts_type(parent);
        }
        self.push(
            AstNode::Unknown(UnknownNode {
                span: Span::new(start, self.prev_end),
            }),
            parent,
        )
    }

    // --- TypeScript declarations ---

    /// Parse `type Name = Type`.
    pub(crate) fn parse_ts_type_alias(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let alias_id = self.reserve(parent);
        self.bump(); // `type`

        let id = self.parse_binding_identifier(Some(alias_id));

        let type_parameters = if self.at(TokenKind::LAngle) {
            self.parse_ts_type_parameters(Some(alias_id))
        } else {
            Vec::new()
        };

        let _ = self.expect(TokenKind::Eq);
        let type_annotation = Some(self.parse_ts_type(Some(alias_id)));
        self.expect_semicolon();

        self.tree.set(
            alias_id,
            AstNode::TSTypeAliasDeclaration(TSTypeAliasDeclarationNode {
                span: Span::new(start, self.prev_end),
                id,
                type_parameters: type_parameters.into_boxed_slice(),
                type_annotation,
            }),
        );
        alias_id
    }

    /// Parse `interface Name { ... }`.
    pub(crate) fn parse_ts_interface(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let iface_id = self.reserve(parent);
        self.bump(); // `interface`

        let id = self.parse_binding_identifier(Some(iface_id));

        // Optional type parameters
        let _type_params = if self.at(TokenKind::LAngle) {
            self.parse_ts_type_parameters(Some(iface_id))
        } else {
            Vec::new()
        };

        // Optional `extends`
        if self.eat(TokenKind::Extends) {
            loop {
                let _ = self.parse_ts_type(Some(iface_id));
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }

        // Body
        let _ = self.expect(TokenKind::LBrace);
        let mut body = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let member = self.parse_ts_type_member(Some(iface_id));
            body.push(member);
        }
        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            iface_id,
            AstNode::TSInterfaceDeclaration(TSInterfaceDeclarationNode {
                span: Span::new(start, end),
                id,
                body: body.into_boxed_slice(),
            }),
        );
        iface_id
    }

    /// Parse `enum Name { ... }`.
    pub(crate) fn parse_ts_enum(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_ts_enum_inner(parent, false)
    }

    /// Parse `const enum` (called after `const` is already consumed).
    pub(crate) fn parse_ts_const_enum(
        &mut self,
        parent: Option<NodeId>,
        const_start: u32,
    ) -> NodeId {
        self.parse_ts_enum_inner_with_start(parent, true, const_start)
    }

    /// Parse enum body, using current position as start.
    fn parse_ts_enum_inner(&mut self, parent: Option<NodeId>, is_const: bool) -> NodeId {
        self.parse_ts_enum_inner_with_start(parent, is_const, self.start())
    }

    /// Parse enum body with an explicit start position (for `const enum`).
    fn parse_ts_enum_inner_with_start(
        &mut self,
        parent: Option<NodeId>,
        is_const: bool,
        start: u32,
    ) -> NodeId {
        let enum_id = self.reserve(parent);
        self.bump(); // `enum`

        let id = self.parse_binding_identifier(Some(enum_id));

        let _ = self.expect(TokenKind::LBrace);
        let mut members = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let member = self.parse_ts_enum_member(Some(enum_id));
            members.push(member);
            self.eat(TokenKind::Comma);
        }
        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            enum_id,
            AstNode::TSEnumDeclaration(TSEnumDeclarationNode {
                span: Span::new(start, end),
                id,
                members: members.into_boxed_slice(),
                is_const,
                is_declare: false,
            }),
        );
        enum_id
    }

    /// Parse a single enum member.
    fn parse_ts_enum_member(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let member_id = self.reserve(parent);
        let id = self.parse_binding_identifier(Some(member_id));

        let initializer = self
            .eat(TokenKind::Eq)
            .then(|| self.parse_assignment_expression(Some(member_id)));

        self.tree.set(
            member_id,
            AstNode::TSEnumMember(TSEnumMemberNode {
                span: Span::new(start, self.prev_end),
                id,
                initializer,
            }),
        );
        member_id
    }

    /// Parse `namespace Name { ... }` or `module Name { ... }` or `module "name" { ... }`.
    pub(crate) fn parse_ts_module(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let mod_id = self.reserve(parent);
        self.bump(); // `namespace` or `module`

        // Ambient module with string literal name: `module "express" { ... }`
        let id = if self.at(TokenKind::String) {
            let str_start = self.start();
            let raw = self.cur_text();
            let value = if raw.len() >= 2 {
                raw.get(1..raw.len().saturating_sub(1))
                    .unwrap_or_default()
                    .to_owned()
            } else {
                String::new()
            };
            let tok = self.bump();
            self.push(
                AstNode::StringLiteral(starlint_ast::node::StringLiteralNode {
                    span: Span::new(str_start, tok.end),
                    value,
                }),
                Some(mod_id),
            )
        } else {
            self.parse_binding_identifier(Some(mod_id))
        };

        let body = if self.at(TokenKind::LBrace) {
            let block = self.parse_block_statement(Some(mod_id));
            Some(block)
        } else {
            self.expect_semicolon();
            None
        };

        self.tree.set(
            mod_id,
            AstNode::TSModuleDeclaration(TSModuleDeclarationNode {
                span: Span::new(start, self.prev_end),
                id,
                body,
                is_declare: false,
            }),
        );
        mod_id
    }

    /// Parse `declare ...`.
    pub(crate) fn parse_ts_declare(&mut self, parent: Option<NodeId>) -> NodeId {
        self.bump(); // `declare`
        // Parse the declaration that follows
        self.parse_statement_with_parent(parent)
    }

    /// Parse TypeScript `as Type` expressions.
    ///
    /// Called after parsing a binary expression when in TypeScript mode.
    /// Non-null `!` is handled in the call/member chain (left-hand-side).
    pub(crate) fn parse_ts_postfix_expressions(
        &mut self,
        mut expr: NodeId,
        parent: Option<NodeId>,
    ) -> NodeId {
        while self.at(TokenKind::As) && !self.has_preceding_line_break() {
            let start = self.tree.span(expr).map_or(0, |s| s.start);
            self.bump(); // `as`
            let _type_id = self.parse_ts_type(parent);
            let end = self.prev_end;
            let as_id = self.push(
                AstNode::TSAsExpression(TSAsExpressionNode {
                    span: Span::new(start, end),
                    expression: expr,
                }),
                parent,
            );
            expr = as_id;
        }
        expr
    }

    /// Parse an angle-bracket type assertion: `<Type>expr`.
    pub(crate) fn parse_ts_type_assertion(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let assert_id = self.reserve(parent);
        self.bump(); // `<`
        let _type_id = self.parse_ts_type(Some(assert_id));
        let _ = self.expect(TokenKind::RAngle);
        let expression = self.parse_unary_expression(Some(assert_id));
        let end = self.tree.span(expression).map_or(self.prev_end, |s| s.end);
        self.tree.set(
            assert_id,
            AstNode::TSTypeAssertion(TSTypeAssertionNode {
                span: Span::new(start, end),
                expression,
            }),
        );
        assert_id
    }
}
