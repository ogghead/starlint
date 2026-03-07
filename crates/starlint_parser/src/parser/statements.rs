//! Statement and declaration parsing.

use starlint_ast::node::{
    ArrayPatternNode, AssignmentPatternNode, AstNode, BindingIdentifierNode, BlockStatementNode,
    BreakStatementNode, CatchClauseNode, ClassNode, ContinueStatementNode, DebuggerStatementNode,
    DoWhileStatementNode, EmptyStatementNode, ExpressionStatementNode, ForInStatementNode,
    ForOfStatementNode, ForStatementNode, FunctionBodyNode, FunctionNode, IfStatementNode,
    LabeledStatementNode, MethodDefinitionNode, ObjectPatternNode, PropertyDefinitionNode,
    ReturnStatementNode, StaticBlockNode, SwitchCaseNode, SwitchStatementNode, ThrowStatementNode,
    TryStatementNode, VariableDeclarationNode, VariableDeclaratorNode, WhileStatementNode,
    WithStatementNode,
};
use starlint_ast::operator::{MethodDefinitionKind, VariableDeclarationKind};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::Parser;

impl Parser<'_> {
    /// Parse a statement with an explicit parent node.
    pub(crate) fn parse_statement_with_parent(&mut self, parent: Option<NodeId>) -> NodeId {
        match self.cur() {
            TokenKind::LBrace => self.parse_block_statement(parent),
            TokenKind::Var => self.parse_variable_statement(parent),
            TokenKind::Const
                if self.options.typescript && self.peek_next_text() == "other" && {
                    // `const enum` — check if next token is `enum`
                    #[allow(clippy::as_conversions)]
                    let after = self
                        .source
                        .get(self.current.end as usize..)
                        .unwrap_or_default()
                        .trim_start();
                    after.starts_with("enum")
                } =>
            {
                let const_start = self.start();
                self.bump(); // `const`
                self.parse_ts_const_enum(parent, const_start)
            }
            TokenKind::Const | TokenKind::Let => self.parse_lexical_declaration(parent),
            TokenKind::Using => self.parse_using_declaration(parent),
            TokenKind::If => self.parse_if_statement(parent),
            TokenKind::Switch => self.parse_switch_statement(parent),
            TokenKind::For => self.parse_for_statement(parent),
            TokenKind::While => self.parse_while_statement(parent),
            TokenKind::Do => self.parse_do_while_statement(parent),
            TokenKind::Try => self.parse_try_statement(parent),
            TokenKind::Throw => self.parse_throw_statement(parent),
            TokenKind::Return => self.parse_return_statement(parent),
            TokenKind::Break => self.parse_break_statement(parent),
            TokenKind::Continue => self.parse_continue_statement(parent),
            TokenKind::Debugger => self.parse_debugger_statement(parent),
            TokenKind::Semicolon => self.parse_empty_statement(parent),
            TokenKind::With => self.parse_with_statement(parent),
            TokenKind::Function => self.parse_function_declaration(parent),
            TokenKind::Async if self.peek_next_is_function() => {
                self.parse_async_function_declaration(parent)
            }
            TokenKind::Class => self.parse_class_declaration(parent),
            // TypeScript declarations
            TokenKind::Type if self.options.typescript => self.parse_ts_type_alias(parent),
            TokenKind::Interface if self.options.typescript => self.parse_ts_interface(parent),
            TokenKind::Enum if self.options.typescript => self.parse_ts_enum(parent),
            TokenKind::Namespace if self.options.typescript => self.parse_ts_module(parent),
            TokenKind::Module if self.options.typescript => self.parse_ts_module(parent),
            TokenKind::Declare if self.options.typescript => self.parse_ts_declare(parent),
            TokenKind::Abstract if self.options.typescript => {
                // `abstract class ...`
                self.bump(); // skip `abstract`
                self.parse_class_declaration(parent)
            }
            _ => self.parse_expression_or_labeled_statement(parent),
        }
    }

    /// Peek if the next token after the current is `function`.
    pub(crate) fn peek_next_is_function(&self) -> bool {
        // Simplified: check if source after current token starts with "function"
        // This is imprecise but avoids needing a full lookahead buffer.
        #[allow(clippy::as_conversions)]
        let after = self
            .source
            .get(self.current.end as usize..)
            .unwrap_or_default()
            .trim_start();
        after.starts_with("function")
    }

    /// Parse a block statement `{ ... }`.
    pub(crate) fn parse_block_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
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
            AstNode::BlockStatement(BlockStatementNode {
                span: Span::new(start, end),
                body: body.into_boxed_slice(),
            }),
        );
        block_id
    }

    /// Parse a `var` declaration.
    fn parse_variable_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_variable_declaration(parent, VariableDeclarationKind::Var)
    }

    /// Parse a `let` or `const` declaration.
    fn parse_lexical_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        let kind = match self.cur() {
            TokenKind::Const => VariableDeclarationKind::Const,
            _ => VariableDeclarationKind::Let,
        };
        self.parse_variable_declaration(parent, kind)
    }

    /// Parse a `using` declaration.
    fn parse_using_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
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
    fn parse_variable_declarator(&mut self, parent: Option<NodeId>) -> NodeId {
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

    /// Parse an `if` statement.
    fn parse_if_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let if_id = self.reserve(parent);
        self.bump(); // `if`
        let _ = self.expect(TokenKind::LParen);
        let test = self.parse_expression(Some(if_id));
        let _ = self.expect(TokenKind::RParen);
        let consequent = self.parse_statement_as_child(Some(if_id));
        let alternate = self
            .eat(TokenKind::Else)
            .then(|| self.parse_statement_as_child(Some(if_id)));
        let end = alternate
            .and_then(|id| self.tree.span(id))
            .or_else(|| self.tree.span(consequent))
            .map_or(self.prev_end, |s| s.end);
        self.tree.set(
            if_id,
            AstNode::IfStatement(IfStatementNode {
                span: Span::new(start, end),
                test,
                consequent,
                alternate,
            }),
        );
        if_id
    }

    /// Parse a statement as a child node (threads parent through).
    fn parse_statement_as_child(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_statement_with_parent(parent)
    }

    /// Parse a `switch` statement.
    fn parse_switch_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let switch_id = self.reserve(parent);
        self.bump(); // `switch`
        let _ = self.expect(TokenKind::LParen);
        let discriminant = self.parse_expression(Some(switch_id));
        let _ = self.expect(TokenKind::RParen);
        let _ = self.expect(TokenKind::LBrace);

        let mut cases = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let case = self.parse_switch_case(Some(switch_id));
            cases.push(case);
        }

        let end = self.current.end;
        let _ = self.expect(TokenKind::RBrace);

        self.tree.set(
            switch_id,
            AstNode::SwitchStatement(SwitchStatementNode {
                span: Span::new(start, end),
                discriminant,
                cases: cases.into_boxed_slice(),
            }),
        );
        switch_id
    }

    /// Parse a single `case` or `default` clause.
    fn parse_switch_case(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let case_id = self.reserve(parent);

        let test = if self.eat(TokenKind::Default) {
            None
        } else {
            let _ = self.expect(TokenKind::Case);
            Some(self.parse_expression(Some(case_id)))
        };
        let _ = self.expect(TokenKind::Colon);

        let mut consequent = Vec::new();
        while !self.at(TokenKind::Case)
            && !self.at(TokenKind::Default)
            && !self.at(TokenKind::RBrace)
            && !self.at(TokenKind::Eof)
        {
            let stmt = self.parse_statement_list_item(Some(case_id));
            consequent.push(stmt);
        }

        let end = consequent
            .last()
            .and_then(|id| self.tree.span(*id))
            .map_or(self.prev_end, |s| s.end);

        self.tree.set(
            case_id,
            AstNode::SwitchCase(SwitchCaseNode {
                span: Span::new(start, end),
                test,
                consequent: consequent.into_boxed_slice(),
            }),
        );
        case_id
    }

    /// Parse a `for` statement (for, for-in, for-of).
    fn parse_for_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `for`
        let _ = self.expect(TokenKind::LParen);

        // Check for `for (var/let/const ...`
        let for_id = self.reserve(parent);

        if self.at(TokenKind::Semicolon) {
            // `for (; ...)`
            self.bump();
            return self.parse_for_classic(for_id, start, None);
        }

        if self.at(TokenKind::Var) || self.at(TokenKind::Let) || self.at(TokenKind::Const) {
            let kind = match self.cur() {
                TokenKind::Var => VariableDeclarationKind::Var,
                TokenKind::Const => VariableDeclarationKind::Const,
                _ => VariableDeclarationKind::Let,
            };
            let decl_start = self.start();
            self.bump(); // keyword
            let declarator = self.parse_variable_declarator(Some(for_id));

            // Check for `in` or `of`
            if self.at(TokenKind::In) {
                return self.finish_for_in(for_id, start, declarator);
            }
            if self.at(TokenKind::Of) {
                return self.finish_for_of(for_id, start, declarator);
            }

            // Regular for: construct full declaration
            let decl_id = self.reserve(Some(for_id));
            let mut declarators = vec![declarator];
            while self.eat(TokenKind::Comma) {
                declarators.push(self.parse_variable_declarator(Some(decl_id)));
            }
            let _ = self.expect(TokenKind::Semicolon);
            self.tree.set(
                decl_id,
                AstNode::VariableDeclaration(VariableDeclarationNode {
                    span: Span::new(decl_start, self.prev_end),
                    kind,
                    declarations: declarators.into_boxed_slice(),
                }),
            );
            return self.parse_for_classic(for_id, start, Some(decl_id));
        }

        // Expression init
        let init_expr = self.parse_expression(Some(for_id));
        if self.at(TokenKind::In) {
            return self.finish_for_in(for_id, start, init_expr);
        }
        if self.at(TokenKind::Of) {
            return self.finish_for_of(for_id, start, init_expr);
        }
        let _ = self.expect(TokenKind::Semicolon);
        self.parse_for_classic(for_id, start, Some(init_expr))
    }

    /// Finish parsing a classic `for (init; test; update) body`.
    fn parse_for_classic(&mut self, for_id: NodeId, start: u32, init: Option<NodeId>) -> NodeId {
        let test = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression(Some(for_id)))
        };
        let _ = self.expect(TokenKind::Semicolon);
        let update = if self.at(TokenKind::RParen) {
            None
        } else {
            Some(self.parse_expression(Some(for_id)))
        };
        let _ = self.expect(TokenKind::RParen);
        let body = self.parse_statement_with_parent(Some(for_id));
        let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
        self.tree.set(
            for_id,
            AstNode::ForStatement(ForStatementNode {
                span: Span::new(start, end),
                init,
                test,
                update,
                body,
            }),
        );
        for_id
    }

    /// Finish a `for ... in` loop.
    fn finish_for_in(&mut self, for_id: NodeId, start: u32, left: NodeId) -> NodeId {
        self.bump(); // `in`
        let right = self.parse_expression(Some(for_id));
        let _ = self.expect(TokenKind::RParen);
        let body = self.parse_statement_with_parent(Some(for_id));
        let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
        self.tree.set(
            for_id,
            AstNode::ForInStatement(ForInStatementNode {
                span: Span::new(start, end),
                left,
                right,
                body,
            }),
        );
        for_id
    }

    /// Finish a `for ... of` loop.
    fn finish_for_of(&mut self, for_id: NodeId, start: u32, left: NodeId) -> NodeId {
        self.bump(); // `of`
        let right = self.parse_expression(Some(for_id));
        let _ = self.expect(TokenKind::RParen);
        let body = self.parse_statement_with_parent(Some(for_id));
        let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
        self.tree.set(
            for_id,
            AstNode::ForOfStatement(ForOfStatementNode {
                span: Span::new(start, end),
                left,
                right,
                body,
                is_await: false,
            }),
        );
        for_id
    }

    /// Parse a `while` statement.
    fn parse_while_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let while_id = self.reserve(parent);
        self.bump(); // `while`
        let _ = self.expect(TokenKind::LParen);
        let test = self.parse_expression(Some(while_id));
        let _ = self.expect(TokenKind::RParen);
        let body = self.parse_statement_with_parent(Some(while_id));
        let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
        self.tree.set(
            while_id,
            AstNode::WhileStatement(WhileStatementNode {
                span: Span::new(start, end),
                test,
                body,
            }),
        );
        while_id
    }

    /// Parse a `do ... while` statement.
    fn parse_do_while_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let do_id = self.reserve(parent);
        self.bump(); // `do`
        let body = self.parse_statement_with_parent(Some(do_id));
        let _ = self.expect(TokenKind::While);
        let _ = self.expect(TokenKind::LParen);
        let test = self.parse_expression(Some(do_id));
        let _ = self.expect(TokenKind::RParen);
        self.expect_semicolon();
        let end = self.prev_end;
        self.tree.set(
            do_id,
            AstNode::DoWhileStatement(DoWhileStatementNode {
                span: Span::new(start, end),
                body,
                test,
            }),
        );
        do_id
    }

    /// Parse a `try ... catch ... finally` statement.
    fn parse_try_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let try_id = self.reserve(parent);
        self.bump(); // `try`
        let block = self.parse_block_statement(Some(try_id));

        let handler = self.at(TokenKind::Catch).then(|| {
            let catch_start = self.start();
            let catch_id = self.reserve(Some(try_id));
            self.bump(); // `catch`
            let param = self.eat(TokenKind::LParen).then(|| {
                let p = self.parse_binding_pattern(Some(catch_id));
                let _ = self.expect(TokenKind::RParen);
                p
            });
            let body = self.parse_block_statement(Some(catch_id));
            let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
            self.tree.set(
                catch_id,
                AstNode::CatchClause(CatchClauseNode {
                    span: Span::new(catch_start, end),
                    param,
                    body,
                }),
            );
            catch_id
        });

        let finalizer = self
            .eat(TokenKind::Finally)
            .then(|| self.parse_block_statement(Some(try_id)));

        let end = finalizer
            .and_then(|id| self.tree.span(id))
            .or_else(|| handler.and_then(|id| self.tree.span(id)))
            .map_or(self.prev_end, |s| s.end);

        self.tree.set(
            try_id,
            AstNode::TryStatement(TryStatementNode {
                span: Span::new(start, end),
                block,
                handler,
                finalizer,
            }),
        );
        try_id
    }

    /// Parse a `throw` statement.
    fn parse_throw_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let throw_id = self.reserve(parent);
        self.bump(); // `throw`
        let argument = self.parse_expression(Some(throw_id));
        self.expect_semicolon();
        let end = self.prev_end;
        self.tree.set(
            throw_id,
            AstNode::ThrowStatement(ThrowStatementNode {
                span: Span::new(start, end),
                argument,
            }),
        );
        throw_id
    }

    /// Parse a `return` statement.
    fn parse_return_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let ret_id = self.reserve(parent);
        self.bump(); // `return`

        let argument = if self.at(TokenKind::Semicolon)
            || self.at(TokenKind::RBrace)
            || self.at(TokenKind::Eof)
            || self.has_preceding_line_break()
        {
            None
        } else {
            Some(self.parse_expression(Some(ret_id)))
        };
        self.expect_semicolon();
        let end = self.prev_end;
        self.tree.set(
            ret_id,
            AstNode::ReturnStatement(ReturnStatementNode {
                span: Span::new(start, end),
                argument,
            }),
        );
        ret_id
    }

    /// Parse a `break` statement.
    fn parse_break_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `break`
        let label = (!self.at(TokenKind::Semicolon)
            && !self.at(TokenKind::RBrace)
            && !self.has_preceding_line_break()
            && self.at(TokenKind::Identifier))
        .then(|| self.cur_text().to_owned());
        if label.is_some() {
            self.bump();
        }
        self.expect_semicolon();
        self.push(
            AstNode::BreakStatement(BreakStatementNode {
                span: Span::new(start, self.prev_end),
                label,
            }),
            parent,
        )
    }

    /// Parse a `continue` statement.
    fn parse_continue_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `continue`
        let label = (!self.at(TokenKind::Semicolon)
            && !self.at(TokenKind::RBrace)
            && !self.has_preceding_line_break()
            && self.at(TokenKind::Identifier))
        .then(|| self.cur_text().to_owned());
        if label.is_some() {
            self.bump();
        }
        self.expect_semicolon();
        self.push(
            AstNode::ContinueStatement(ContinueStatementNode {
                span: Span::new(start, self.prev_end),
                label,
            }),
            parent,
        )
    }

    /// Parse a `debugger` statement.
    fn parse_debugger_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `debugger`
        self.expect_semicolon();
        self.push(
            AstNode::DebuggerStatement(DebuggerStatementNode {
                span: Span::new(start, self.prev_end),
            }),
            parent,
        )
    }

    /// Parse an empty statement (`;`).
    fn parse_empty_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        self.bump(); // `;`
        self.push(
            AstNode::EmptyStatement(EmptyStatementNode {
                span: Span::new(start, self.prev_end),
            }),
            parent,
        )
    }

    /// Parse a `with` statement.
    fn parse_with_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let with_id = self.reserve(parent);
        self.bump(); // `with`
        let _ = self.expect(TokenKind::LParen);
        let object = self.parse_expression(Some(with_id));
        let _ = self.expect(TokenKind::RParen);
        let body = self.parse_statement_with_parent(Some(with_id));
        let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
        self.tree.set(
            with_id,
            AstNode::WithStatement(WithStatementNode {
                span: Span::new(start, end),
                object,
                body,
            }),
        );
        with_id
    }

    /// Parse an expression statement or labeled statement.
    fn parse_expression_or_labeled_statement(&mut self, parent: Option<NodeId>) -> NodeId {
        let start = self.start();
        let expr = self.parse_expression(parent);

        // Check for label: `identifier:`
        if self.at(TokenKind::Colon) {
            if let Some(AstNode::IdentifierReference(ident)) = self.tree.get(expr) {
                let label_name = ident.name.clone();
                self.bump(); // `:`
                let label_id = self.reserve(parent);
                let body = self.parse_statement_with_parent(Some(label_id));
                let end = self.tree.span(body).map_or(self.prev_end, |s| s.end);
                self.tree.set(
                    label_id,
                    AstNode::LabeledStatement(LabeledStatementNode {
                        span: Span::new(start, end),
                        label: label_name,
                        body,
                    }),
                );
                return label_id;
            }
        }

        self.expect_semicolon();
        let end = self.prev_end;
        let expr_stmt_id = self.reserve(parent);
        self.tree.set(
            expr_stmt_id,
            AstNode::ExpressionStatement(ExpressionStatementNode {
                span: Span::new(start, end),
                expression: expr,
            }),
        );
        expr_stmt_id
    }

    // --- Function / Class ---

    /// Parse a function declaration.
    pub(crate) fn parse_function_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
        self.parse_function(parent, false)
    }

    /// Parse an async function declaration.
    fn parse_async_function_declaration(&mut self, parent: Option<NodeId>) -> NodeId {
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

        let end = body
            .and_then(|id| self.tree.span(id))
            .map_or(self.prev_end, |s| s.end);

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

    /// Peek at the text of the next token (after current) without consuming.
    pub(crate) fn peek_next_text(&self) -> &str {
        #[allow(clippy::as_conversions)]
        let after = self
            .source
            .get(self.current.end as usize..)
            .unwrap_or_default()
            .trim_start();
        if let Some(first) = after.chars().next() {
            match first {
                '(' => "(",
                ';' => ";",
                '=' => "=",
                '{' => "{",
                '}' => "}",
                ',' => ",",
                ':' => ":",
                _ => "other",
            }
        } else {
            ""
        }
    }
}
