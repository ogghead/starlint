//! Variable, function, and class declarations.

use starlint_ast::node::{
    ArrayPatternNode, AssignmentPatternNode, AstNode, BindingIdentifierNode, ClassNode,
    FunctionBodyNode, FunctionNode, MethodDefinitionNode, ObjectPatternNode,
    PropertyDefinitionNode, StaticBlockNode, VariableDeclarationNode, VariableDeclaratorNode,
};
use starlint_ast::operator::{MethodDefinitionKind, VariableDeclarationKind};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    /// Parse a `var` declaration.
    pub(super) fn parse_variable_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_variable_declaration(parent, VariableDeclarationKind::Var)
    }

    /// Parse a `let` or `const` declaration.
    pub(super) fn parse_lexical_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        let kind = match self.cur() {
            TokenKind::Const => VariableDeclarationKind::Const,
            _ => VariableDeclarationKind::Let,
        };
        self.parse_variable_declaration(parent, kind)
    }

    /// Parse a `using` declaration.
    pub(super) fn parse_using_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        // Check for `await using`
        let kind = VariableDeclarationKind::Using;
        self.parse_variable_declaration(parent, kind)
    }

    /// Parse a variable declaration (var/let/const/using).
    pub(crate) fn parse_variable_declaration(
        &mut self,
        parent: Option<NodeId>,
        kind: VariableDeclarationKind,
    ) -> NodeId {
        let start = self.start();
        let decl_id = self.reserve(parent);
        self.bump(); // consume keyword

        let mut declarations = Vec::new();
        loop {
            let declarator = self.parse_variable_declarator(Some(decl_id));
            declarations.push(declarator);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        self.expect_semicolon();

        let end = self.prev_end;
        self.tree.set(
            decl_id,
            AstNode::VariableDeclaration(VariableDeclarationNode {
                span: Span::new(start, end),
                kind,
                declarations: declarations.into_boxed_slice(),
            }),
        );
        decl_id
    }

    /// Parse a single variable declarator.
    pub(crate) fn parse_variable_declarator(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let decl_id = self.reserve(parent);

        let id = self.parse_binding_pattern(Some(decl_id));

        // Optional TS type annotation
        let type_annotation = (self.options.typescript && self.at(TokenKind::Colon)).then(|| {
            self.bump();
            self.parse_ts_type(Some(decl_id))
        });

        let init = self
            .eat(TokenKind::Eq)
            .then(|| self.parse_assignment_expression(Some(decl_id)));

        let end = init
            .and_then(|id| self.tree.span(id))
            .or_else(|| type_annotation.and_then(|id| self.tree.span(id)))
            .or_else(|| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end);

        self.tree.set(
            decl_id,
            AstNode::VariableDeclarator(VariableDeclaratorNode {
                span: Span::new(start, end),
                id,
                type_annotation,
                init,
            }),
        );
        decl_id
    }

    /// Parse a binding pattern (identifier, array destructuring, or object destructuring).
    pub(crate) fn parse_binding_pattern(&mut self, parent: Option<NodeId>) -> NodeId {
        match self.cur() {
            TokenKind::LBracket => self.parse_array_pattern(parent),
            TokenKind::LBrace => self.parse_object_pattern(parent),
            _ => self.parse_binding_identifier(parent),
        }
    }

    /// Parse a binding identifier.
    pub(crate) fn parse_binding_identifier(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let name = self.cur_text().to_owned();
        let tok = self.bump();
        self.push(
            AstNode::BindingIdentifier(BindingIdentifierNode {
                span: Span::new(start, tok.end),
                name,
            }),
            parent,
        )
    }

    /// Parse array destructuring pattern.
    fn parse_array_pattern(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let pat_id = self.reserve(parent);
        self.bump(); // `[`

        let mut elements = Vec::new();
        let mut rest = None;

        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Comma) {
                elements.push(None);
                self.bump();
                continue;
            }
            if self.at(TokenKind::DotDotDot) {
                self.bump();
                rest = Some(self.parse_binding_pattern(Some(pat_id)));
                break;
            }
            let elem = self.parse_binding_element(Some(pat_id));
            elements.push(Some(elem));
            if !self.at(TokenKind::RBracket) {
                self.eat(TokenKind::Comma);
            }
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBracket);

        self.tree.set(
            pat_id,
            AstNode::ArrayPattern(ArrayPatternNode {
                span: Span::new(start, end),
                elements: elements.into_boxed_slice(),
                rest,
            }),
        );
        pat_id
    }

    /// Parse object destructuring pattern.
    fn parse_object_pattern(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let pat_id = self.reserve(parent);
        self.bump(); // `{`

        let mut properties = Vec::new();
        let mut rest = None;

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::DotDotDot) {
                self.bump();
                rest = Some(self.parse_binding_pattern(Some(pat_id)));
                break;
            }
            let prop = self.parse_binding_property(Some(pat_id));
            properties.push(prop);
            if !self.at(TokenKind::RBrace) {
                self.eat(TokenKind::Comma);
            }
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            pat_id,
            AstNode::ObjectPattern(ObjectPatternNode {
                span: Span::new(start, end),
                properties: properties.into_boxed_slice(),
                rest,
            }),
        );
        pat_id
    }

    /// Parse a binding element (pattern with optional default).
    fn parse_binding_element(&mut self, parent: Option<NodeId>) -> NodeId {
        let pattern = self.parse_binding_pattern(parent);

        if self.eat(TokenKind::Eq) {
            let start = self.tree.span(pattern).map_or(0, |s| s.start);
            let assign_id = self.reserve(parent);
            let right = self.parse_assignment_expression(Some(assign_id));
            let end = self.tree.span(right).map_or(0, |s| s.end);
            self.tree.set(
                assign_id,
                AstNode::AssignmentPattern(AssignmentPatternNode {
                    span: Span::new(start, end),
                    left: pattern,
                    right,
                }),
            );
            return assign_id;
        }

        pattern
    }

    /// Parse a binding property in an object pattern.
    fn parse_binding_property(&mut self, parent: Option<NodeId>) -> NodeId {
        // Similar to object property but for patterns
        let start = self.start();
        let prop_id = self.reserve(parent);
        let key_text = self.cur_text().to_owned();
        let key_start = self.start();
        let key_tok = self.bump();

        let key = self.push(
            AstNode::BindingIdentifier(BindingIdentifierNode {
                span: Span::new(key_start, key_tok.end),
                name: key_text,
            }),
            Some(prop_id),
        );

        if self.at(TokenKind::Colon) {
            // `key: pattern`
            self.bump();
            let value = self.parse_binding_element(Some(prop_id));
            let end = self.tree.span(value).map_or(0, |s| s.end);
            self.tree.set(
                prop_id,
                AstNode::VariableDeclarator(VariableDeclaratorNode {
                    span: Span::new(start, end),
                    id: value,
                    type_annotation: None,
                    init: None,
                }),
            );
        } else if self.at(TokenKind::Eq) {
            // `key = default`
            self.bump();
            let default_val = self.parse_assignment_expression(Some(prop_id));
            let end = self.tree.span(default_val).map_or(0, |s| s.end);
            let assign = self.push(
                AstNode::AssignmentPattern(AssignmentPatternNode {
                    span: Span::new(start, end),
                    left: key,
                    right: default_val,
                }),
                Some(prop_id),
            );
            self.tree.set(
                prop_id,
                AstNode::VariableDeclarator(VariableDeclaratorNode {
                    span: Span::new(start, end),
                    id: assign,
                    type_annotation: None,
                    init: None,
                }),
            );
        } else {
            // Shorthand: `{ x }` — key is also value
            self.tree.set(
                prop_id,
                AstNode::VariableDeclarator(VariableDeclaratorNode {
                    span: Span::new(start, key_tok.end),
                    id: key,
                    type_annotation: None,
                    init: None,
                }),
            );
        }

        prop_id
    }

    // --- Function / Class ---

    /// Parse a function declaration.
    pub(crate) fn parse_function_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_function(parent, false)
    }

    /// Parse an async function declaration.
    pub(super) fn parse_async_function_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        self.bump(); // `async`
        self.parse_function(parent, true)
    }

    /// Parse a function (declaration or expression).
    pub(crate) fn parse_function(&mut self, parent: Option<NodeId>, is_async: bool) -> NodeId {
        let start = if is_async {
            // async was already consumed, use its span
            self.prev_end.saturating_sub(5) // approximate
        } else {
            self.start()
        };
        let func_id = self.reserve(parent);
        self.bump(); // `function`

        let is_generator = self.eat(TokenKind::Star);

        // Optional function name
        let id = (self.at(TokenKind::Identifier) || self.cur().is_keyword())
            .then(|| self.parse_binding_identifier(Some(func_id)));

        // Optional type parameters
        let type_parameters = if self.options.typescript && self.at(TokenKind::LAngle) {
            self.parse_ts_type_parameters(Some(func_id))
        } else {
            Vec::new()
        };

        // Parameters
        let params = self.parse_formal_parameters(Some(func_id));

        // Optional return type
        let return_type = (self.options.typescript && self.at(TokenKind::Colon)).then(|| {
            self.bump();
            self.parse_ts_type(Some(func_id))
        });

        // Body
        let body = if self.at(TokenKind::LBrace) {
            Some(self.parse_function_body(Some(func_id)))
        } else {
            // Abstract method or declaration without body
            self.expect_semicolon();
            None
        };

        let end = body
            .and_then(|id| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end);

        self.tree.set(
            func_id,
            AstNode::Function(FunctionNode {
                span: Span::new(start, end),
                id,
                is_async,
                is_generator,
                is_declare: false,
                type_parameters: type_parameters.into_boxed_slice(),
                params: params.into_boxed_slice(),
                return_type,
                body,
            }),
        );
        func_id
    }

    /// Parse a function expression.
    pub(crate) fn parse_function_expression(
        &mut self,
        parent: Option<NodeId>,
        is_async: bool,
    ) -> NodeId {
        let start = self.start();
        let func_id = self.reserve(parent);

        if is_async {
            // `async` was not consumed yet in this path
        }
        self.bump(); // `function`

        let is_generator = self.eat(TokenKind::Star);

        let id = (self.at(TokenKind::Identifier)
            || (self.cur().is_keyword() && !self.at(TokenKind::LParen)))
        .then(|| self.parse_binding_identifier(Some(func_id)));

        let type_parameters = if self.options.typescript && self.at(TokenKind::LAngle) {
            self.parse_ts_type_parameters(Some(func_id))
        } else {
            Vec::new()
        };

        let params = self.parse_formal_parameters(Some(func_id));

        let return_type = (self.options.typescript && self.at(TokenKind::Colon)).then(|| {
            self.bump();
            self.parse_ts_type(Some(func_id))
        });

        let body = self
            .at(TokenKind::LBrace)
            .then(|| self.parse_function_body(Some(func_id)));

        // During error recovery start may exceed prev_end (e.g. at EOF);
        // clamp so the span is always valid.
        let end = body
            .and_then(|id| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end)
            .max(start);

        self.tree.set(
            func_id,
            AstNode::Function(FunctionNode {
                span: Span::new(start, end),
                id,
                is_async,
                is_generator,
                is_declare: false,
                type_parameters: type_parameters.into_boxed_slice(),
                params: params.into_boxed_slice(),
                return_type,
                body,
            }),
        );
        func_id
    }

    /// Parse a function expression body only (for method shorthands).
    pub(crate) fn parse_function_expression_body(
        &mut self,
        parent: Option<NodeId>,
        is_async: bool,
        is_generator: bool,
    ) -> NodeId {
        let start = self.start();
        let func_id = self.reserve(parent);
        let params = self.parse_formal_parameters(Some(func_id));

        let return_type = (self.options.typescript && self.at(TokenKind::Colon)).then(|| {
            self.bump();
            self.parse_ts_type(Some(func_id))
        });

        let body = self
            .at(TokenKind::LBrace)
            .then(|| self.parse_function_body(Some(func_id)));

        // During error recovery start may exceed prev_end (e.g. at EOF);
        // clamp so the span is always valid.
        let end = body
            .and_then(|id| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end)
            .max(start);

        self.tree.set(
            func_id,
            AstNode::Function(FunctionNode {
                span: Span::new(start, end),
                id: None,
                is_async,
                is_generator,
                is_declare: false,
                type_parameters: Box::new([]),
                params: params.into_boxed_slice(),
                return_type,
                body,
            }),
        );
        func_id
    }

    /// Parse formal parameters `(param1, param2, ...)`.
    fn parse_formal_parameters(&mut self, parent: Option<NodeId>) -> Vec<NodeId> {
        let _ = self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::DotDotDot) {
                self.bump();
                let rest = self.parse_binding_pattern(parent);
                // Optional TS type annotation on rest
                if self.options.typescript && self.at(TokenKind::Colon) {
                    self.bump();
                    let _type_ann = self.parse_ts_type(parent);
                }
                params.push(rest);
                break;
            }
            let param = self.parse_binding_element(parent);
            // Optional TS type annotation
            if self.options.typescript && self.at(TokenKind::Colon) {
                self.bump();
                let _type_ann = self.parse_ts_type(parent);
            }
            params.push(param);
            if !self.at(TokenKind::RParen) {
                let _ = self.expect(TokenKind::Comma);
            }
        }
        let _ = self.expect(TokenKind::RParen);
        params
    }

    /// Parse a function body `{ statements }`.
    pub(crate) fn parse_function_body(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let body_id = self.reserve(parent);
        let _ = self.expect(TokenKind::LBrace);

        let mut stmts = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let stmt = self.parse_statement_list_item(Some(body_id));
            stmts.push(stmt);
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            body_id,
            AstNode::FunctionBody(FunctionBodyNode {
                span: Span::new(start, end),
                statements: stmts.into_boxed_slice(),
            }),
        );
        body_id
    }

    /// Parse a class declaration.
    pub(crate) fn parse_class_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_class(parent, false)
    }

    /// Parse a class expression.
    pub(crate) fn parse_class_expression(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_class(parent, true)
    }

    /// Parse a class.
    fn parse_class(&mut self, parent: Option<NodeId>, _is_expression: bool) -> NodeId {
        let start = self.start();
        let class_id = self.reserve(parent);
        self.bump(); // `class`

        // Optional name
        let id = (self.at(TokenKind::Identifier)
            || (self.cur().is_keyword()
                && !self.at(TokenKind::Extends)
                && !self.at(TokenKind::LBrace)))
        .then(|| self.parse_binding_identifier(Some(class_id)));

        // Optional `extends`
        let super_class = self
            .eat(TokenKind::Extends)
            .then(|| self.parse_left_hand_side_expression(Some(class_id)));

        // Optional `implements`
        if self.options.typescript && self.at(TokenKind::Implements) {
            self.bump();
            // Skip implements list
            loop {
                let _ = self.parse_ts_type(Some(class_id));
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }

        let _ = self.expect(TokenKind::LBrace);

        let mut body_members = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Semicolon) {
                self.bump();
                continue;
            }
            let member = self.parse_class_member(Some(class_id));
            body_members.push(member);
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            class_id,
            AstNode::Class(ClassNode {
                span: Span::new(start, end),
                id,
                super_class,
                is_declare: false,
                is_abstract: false,
                body: body_members.into_boxed_slice(),
            }),
        );
        class_id
    }

    /// Parse a class member (method, property, static block).
    fn parse_class_member(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();

        // Handle `static { ... }` (static blocks)
        if self.at(TokenKind::Static) {
            // Check if next is `{` for static block
            #[allow(clippy::as_conversions)]
            let after = self
                .source
                .get(self.current.end as usize..)
                .unwrap_or_default()
                .trim_start();
            if after.starts_with('{') {
                self.bump(); // `static`
                let block_id = self.reserve(parent);
                let _ = self.expect(TokenKind::LBrace);
                let mut body = Vec::new();
                while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                    let stmt = self.parse_statement_list_item(Some(block_id));
                    body.push(stmt);
                }
                let end = self.current.end;
                let _ = self.expect(TokenKind::RBrace);
                self.tree.set(
                    block_id,
                    AstNode::StaticBlock(StaticBlockNode {
                        span: Span::new(start, end),
                        body: body.into_boxed_slice(),
                    }),
                );
                return block_id;
            }
        }

        // Parse modifiers: static, abstract, override, readonly, accessor
        let is_static = self.eat(TokenKind::Static);
        let _is_abstract = self.options.typescript && self.eat(TokenKind::Abstract);
        let _is_override = self.options.typescript && self.eat(TokenKind::Override);
        let _is_readonly = self.options.typescript && self.eat(TokenKind::Readonly);

        // Check for getter/setter
        let kind = if self.at(TokenKind::Get) {
            let after_text = self.peek_next_text();
            if after_text != "(" && after_text != ";" && after_text != "=" {
                self.bump();
                MethodDefinitionKind::Get
            } else {
                MethodDefinitionKind::Method
            }
        } else if self.at(TokenKind::Set) {
            let after_text = self.peek_next_text();
            if after_text != "(" && after_text != ";" && after_text != "=" {
                self.bump();
                MethodDefinitionKind::Set
            } else {
                MethodDefinitionKind::Method
            }
        } else {
            MethodDefinitionKind::Method
        };

        let is_generator = self.eat(TokenKind::Star);
        let is_async = self.eat(TokenKind::Async);

        // Parse key
        let computed = self.at(TokenKind::LBracket);
        let member_id = self.reserve(parent);
        let key = if computed {
            self.bump(); // `[`
            let k = self.parse_assignment_expression(Some(member_id));
            let _ = self.expect(TokenKind::RBracket);
            k
        } else {
            self.parse_primary_expression(Some(member_id))
        };

        // Check for `constructor`
        let actual_kind = if let Some(AstNode::IdentifierReference(ident)) = self.tree.get(key) {
            if ident.name == "constructor" && kind == MethodDefinitionKind::Method {
                MethodDefinitionKind::Constructor
            } else {
                kind
            }
        } else {
            kind
        };

        // Method or property?
        if self.at(TokenKind::LParen) || is_generator {
            // Method
            let value =
                self.parse_function_expression_body(Some(member_id), is_async, is_generator);
            let end = self.tree.span(value).map_or(self.prev_end, |s| s.end);
            self.tree.set(
                member_id,
                AstNode::MethodDefinition(MethodDefinitionNode {
                    span: Span::new(start, end),
                    key,
                    value,
                    kind: actual_kind,
                    computed,
                    is_static,
                    is_accessor: false,
                }),
            );
        } else {
            // Property
            // Optional TS type annotation
            if self.options.typescript && self.at(TokenKind::Colon) {
                self.bump();
                let _type_ann = self.parse_ts_type(Some(member_id));
            }
            let value = self
                .eat(TokenKind::Eq)
                .then(|| self.parse_assignment_expression(Some(member_id)));
            self.expect_semicolon();
            let end = self.prev_end;
            self.tree.set(
                member_id,
                AstNode::PropertyDefinition(PropertyDefinitionNode {
                    span: Span::new(start, end),
                    key,
                    value,
                    computed,
                    is_static,
                    is_declare: false,
                }),
            );
        }

        member_id
    }
}
