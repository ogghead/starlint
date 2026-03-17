//! Control flow statement parsing: if/else, switch/case, try/catch/finally, throw, return, break, continue.

use starlint_ast::node::{
    AstNode, BreakStatementNode, CatchClauseNode, ContinueStatementNode, DebuggerStatementNode,
    IfStatementNode, ReturnStatementNode, SwitchCaseNode, SwitchStatementNode, ThrowStatementNode,
    TryStatementNode,
};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    /// Parse an `if` statement.
    pub(super) fn parse_if_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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

    /// Parse a `switch` statement.
    pub(super) fn parse_switch_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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

    /// Parse a `try ... catch ... finally` statement.
    pub(super) fn parse_try_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_throw_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_return_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_break_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_continue_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_debugger_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
}
