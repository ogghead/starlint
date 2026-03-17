//! Block, labeled, empty, with, and expression statements.

use starlint_ast::node::{
    AstNode, BlockStatementNode, EmptyStatementNode, ExpressionStatementNode, LabeledStatementNode,
    WithStatementNode,
};
use starlint_ast::types::{NodeId, Span};

use crate::token::TokenKind;

use super::super::Parser;

impl Parser<'_> {
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

    /// Parse an empty statement (`;`).
    pub(super) fn parse_empty_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_with_statement(&mut self, parent: Option<NodeId>) -> NodeId {
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
    pub(super) fn parse_expression_or_labeled_statement(
        &mut self,
        parent: Option<NodeId>,
    ) -> NodeId {
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
}
