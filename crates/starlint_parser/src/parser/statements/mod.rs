//! Statement and declaration parsing.

pub(in crate::parser) mod blocks;
pub(in crate::parser) mod control_flow;
pub(in crate::parser) mod declarations;
pub(in crate::parser) mod loops;

use starlint_ast::types::NodeId;

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

    /// Parse a statement as a child node (threads parent through).
    pub(in crate::parser::statements) fn parse_statement_as_child(
        &mut self,
        parent: Option<NodeId>,
    ) -> NodeId {
        self.parse_statement_with_parent(parent)
    }

    /// Parse a statement list item (statement or declaration).
    pub(in crate::parser) fn parse_statement_list_item(
        &mut self,
        parent: Option<NodeId>,
    ) -> NodeId {
        // Module declarations
        if self.options.module {
            match self.cur() {
                TokenKind::Import => return self.parse_import_declaration(parent),
                TokenKind::Export => return self.parse_export_declaration(parent),
                _ => {}
            }
        }
        self.parse_statement_with_parent(parent)
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
