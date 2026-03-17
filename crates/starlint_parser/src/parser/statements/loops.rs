//! Loop statement parsing: for, for-in, for-of, while, do-while.

use starlint_ast::node::{
    AstNode, DoWhileStatementNode, ForInStatementNode, ForOfStatementNode, ForStatementNode,
    VariableDeclarationNode, WhileStatementNode,
};
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    /// Parse a `for` statement (for, for-in, for-of).
    pub(super) fn parse_for_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_while_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_do_while_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
}
