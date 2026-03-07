//! Recursive descent parser for JS/TS/JSX.
//!
//! Constructs an [`AstTree`] directly during parsing using the reserve-then-set
//! pattern: reserve a slot for the parent node, parse children (which push into
//! the tree), then set the parent node with child `NodeId` references.

mod expressions;
mod jsx;
mod modules;
mod statements;
mod typescript;

use crate::ParseOptions;
use crate::ParseResult;
use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

use starlint_ast::node::{AstNode, ProgramNode};
use starlint_ast::tree::AstTree;
use starlint_ast::types::{NodeId, Span};

/// The parser state.
pub struct Parser<'a> {
    /// The source text (for extracting spans/text).
    source: &'a str,
    /// The lexer producing tokens.
    lexer: Lexer<'a>,
    /// Current token.
    current: Token,
    /// Previous token (for span tracking).
    prev_end: u32,
    /// The AST being built.
    pub(crate) tree: AstTree,
    /// Parse options (JSX, TypeScript, module mode).
    options: ParseOptions,
    /// Collected parse errors.
    errors: Vec<ParseError>,
    /// Whether the parser entered panic recovery mode.
    panicked: bool,
    /// Set to `true` when an `async` prefix was consumed for an arrow function.
    pending_async: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser.
    pub fn new(source: &'a str, options: ParseOptions) -> Self {
        let mut lexer = Lexer::new(source.as_bytes());
        let current = lexer.next_token();
        Self {
            source,
            lexer,
            current,
            prev_end: 0,
            tree: AstTree::with_capacity(256),
            options,
            errors: Vec::new(),
            panicked: false,
            pending_async: false,
        }
    }

    /// Parse the entire source into a [`ParseResult`].
    pub fn parse(&mut self) -> ParseResult {
        let program_id = self.tree.reserve(None);
        let start = self.current.start;

        let mut body = Vec::new();
        while !self.at(TokenKind::Eof) {
            let stmt = self.parse_statement_list_item(Some(program_id));
            body.push(stmt);
        }

        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let end = self.source.len() as u32;
        self.tree.set(
            program_id,
            AstNode::Program(ProgramNode {
                span: Span::new(start, end),
                is_module: self.options.module,
                body: body.into_boxed_slice(),
            }),
        );

        let mut errors = std::mem::take(&mut self.errors);
        errors.extend(self.lexer.take_errors());

        ParseResult {
            tree: std::mem::take(&mut self.tree),
            errors,
            panicked: self.panicked,
        }
    }

    // --- Token helpers ---

    /// Check if the current token matches a kind.
    pub(crate) fn at(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }

    /// Check if the current token matches any of the given kinds.
    #[allow(dead_code)]
    pub(crate) fn at_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.current.kind)
    }

    /// Get the current token kind.
    pub(crate) const fn cur(&self) -> TokenKind {
        self.current.kind
    }

    /// Get the current token's start offset.
    pub(crate) const fn start(&self) -> u32 {
        self.current.start
    }

    /// Get the source text for a span.
    pub(crate) fn text(&self, start: u32, end: u32) -> &'a str {
        #[allow(clippy::as_conversions)]
        self.source
            .get(start as usize..end as usize)
            .unwrap_or_default()
    }

    /// Get the source text for the current token.
    pub(crate) fn cur_text(&self) -> &'a str {
        self.text(self.current.start, self.current.end)
    }

    /// Advance to the next token, returning the consumed token.
    pub(crate) fn bump(&mut self) -> Token {
        let token = self.current.clone();
        self.prev_end = token.end;
        self.current = self.lexer.next_token();
        token
    }

    /// Consume the current token if it matches `kind`, otherwise return an error.
    pub(crate) fn expect(&mut self, kind: TokenKind) -> Result<Token, ()> {
        if self.at(kind) {
            Ok(self.bump())
        } else {
            self.error(format!("expected {kind:?}, found {:?}", self.current.kind));
            Err(())
        }
    }

    /// If the current token matches `kind`, consume it and return `true`.
    pub(crate) fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Report a parse error at the current position.
    pub(crate) fn error(&mut self, message: impl Into<String>) {
        self.panicked = true;
        self.errors.push(ParseError::new(
            message,
            self.current.start,
            self.current.end,
        ));
    }

    /// Reserve a slot in the tree under the given parent.
    pub(crate) fn reserve(&mut self, parent: Option<NodeId>) -> NodeId {
        self.tree.reserve(parent)
    }

    /// Push a node into the tree.
    pub(crate) fn push(&mut self, node: AstNode, parent: Option<NodeId>) -> NodeId {
        self.tree.push(node, parent)
    }

    /// Recover from an error by skipping tokens until we find a sync point.
    #[allow(dead_code)]
    pub(crate) fn recover_to_statement_boundary(&mut self) {
        loop {
            match self.cur() {
                TokenKind::Eof | TokenKind::Semicolon | TokenKind::RBrace => break,
                // These can start a new statement
                TokenKind::Const
                | TokenKind::Let
                | TokenKind::Var
                | TokenKind::Function
                | TokenKind::Class
                | TokenKind::If
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::Switch
                | TokenKind::Try
                | TokenKind::Return
                | TokenKind::Throw
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Import
                | TokenKind::Export => break,
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// Consume a semicolon (explicit or ASI).
    pub(crate) fn expect_semicolon(&mut self) {
        if self.at(TokenKind::Semicolon) {
            self.bump();
        } else if self.at(TokenKind::RBrace) || self.at(TokenKind::Eof) {
            // ASI: implicit semicolon before `}` or EOF
        } else if self.prev_end_on_newline() {
            // ASI: implicit semicolon after line terminator
        } else {
            self.error("expected `;`");
        }
    }

    /// Check if there was a newline between `prev_end` and current token start.
    fn prev_end_on_newline(&self) -> bool {
        #[allow(clippy::as_conversions)]
        let between = self
            .source
            .get(self.prev_end as usize..self.current.start as usize)
            .unwrap_or_default();
        between.contains('\n') || between.contains('\r')
    }

    /// Check if the current token is on a new line compared to the previous token.
    pub(crate) fn has_preceding_line_break(&self) -> bool {
        self.prev_end_on_newline()
    }

    /// Parse a statement list item (statement or declaration).
    fn parse_statement_list_item(&mut self, parent: Option<NodeId>) -> NodeId {
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
}

#[cfg(test)]
mod tests {
    use crate::{ParseOptions, parse};
    use starlint_ast::node::AstNode;
    use starlint_ast::types::NodeId;

    #[test]
    fn parse_empty_program() {
        let result = parse("", ParseOptions::default());
        assert!(result.errors.is_empty(), "no errors for empty program");
        assert_eq!(result.tree.len(), 1, "just the Program node");
        let root = result.tree.get(NodeId::ROOT);
        assert!(
            matches!(root, Some(AstNode::Program(_))),
            "root should be Program"
        );
    }

    #[test]
    fn parse_variable_declaration() {
        let result = parse("const x = 1;", ParseOptions::default());
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(result.tree.len() > 1, "should have more than just Program");
    }

    #[test]
    fn parse_function_declaration() {
        let result = parse("function foo() { return 42; }", ParseOptions::default());
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn parse_if_statement() {
        let result = parse("if (true) { x; } else { y; }", ParseOptions::default());
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }
}
