//! Hand-written lexer for JavaScript/TypeScript/JSX.
//!
//! Operates on a byte slice (`&[u8]`) for performance. Tracks byte offsets
//! for spans. Handles the regex/division ambiguity via parser-supplied context
//! and template literal nesting via an internal depth stack.

use crate::error::ParseError;
use crate::token::{Token, TokenKind, keyword_from_str};

/// Lexer mode — controls how certain characters are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerMode {
    /// Normal JavaScript/TypeScript mode.
    Normal,
    /// Inside a JSX child context (text between tags).
    JsxChild,
    /// Inside a JSX tag (attributes, tag names).
    JsxTag,
}

/// The lexer: scans source bytes into [`Token`]s.
pub struct Lexer<'a> {
    /// Source bytes.
    source: &'a [u8],
    /// Current byte offset.
    pos: usize,
    /// Current lexer mode.
    mode: LexerMode,
    /// Stack of template literal nesting depths.
    /// Each entry is the brace depth at the point `${` was encountered.
    template_depth_stack: Vec<u32>,
    /// Current brace nesting depth (for template literal tracking).
    brace_depth: u32,
    /// Collected parse errors.
    errors: Vec<ParseError>,
    /// Whether the previous token could end an expression.
    /// Used for regex vs division disambiguation.
    prev_token_is_expr_end: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source.
    #[must_use]
    pub const fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            pos: 0,
            mode: LexerMode::Normal,
            template_depth_stack: Vec::new(),
            brace_depth: 0,
            errors: Vec::new(),
            prev_token_is_expr_end: false,
        }
    }

    /// Consume all errors accumulated during lexing.
    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    /// Set the lexer mode (used by the parser for JSX context switching).
    pub const fn set_mode(&mut self, mode: LexerMode) {
        self.mode = mode;
    }

    /// Get the current lexer mode.
    #[must_use]
    pub const fn mode(&self) -> LexerMode {
        self.mode
    }

    /// Current byte position.
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Peek the current byte without advancing.
    #[must_use]
    fn peek(&self) -> Option<u8> {
        self.source.get(self.pos).copied()
    }

    /// Peek the byte at `pos + offset` without advancing.
    #[must_use]
    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.source.get(self.pos.saturating_add(offset)).copied()
    }

    /// Advance by one byte and return the consumed byte.
    fn advance(&mut self) -> Option<u8> {
        let b = self.source.get(self.pos).copied();
        if b.is_some() {
            self.pos = self.pos.saturating_add(1);
        }
        b
    }

    /// Advance by `n` bytes.
    fn advance_by(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.source.len());
    }

    /// Get the source text for a given byte range.
    #[must_use]
    fn source_text(&self, start: usize, end: usize) -> &'a str {
        // The source is assumed to be valid UTF-8 (it was a &str).
        // We use from_utf8_unchecked since the parser was given valid source.
        // However, to be safe, we use from_utf8_lossy in case of issues.
        std::str::from_utf8(self.source.get(start..end).unwrap_or_default()).unwrap_or_default()
    }

    /// Scan the next token.
    #[allow(clippy::too_many_lines)]
    pub fn next_token(&mut self) -> Token {
        match self.mode {
            LexerMode::JsxChild => return self.scan_jsx_child(),
            LexerMode::JsxTag => {
                self.skip_whitespace_and_comments();
                if self.pos >= self.source.len() {
                    return self.make_eof();
                }
                return self.scan_jsx_tag_token();
            }
            LexerMode::Normal => {}
        }

        self.skip_whitespace_and_comments();

        if self.pos >= self.source.len() {
            return self.make_eof();
        }

        // Check if we're returning from a template expression `}`
        if !self.template_depth_stack.is_empty() && self.peek() == Some(b'}') {
            if let Some(&depth) = self.template_depth_stack.last() {
                if self.brace_depth == depth {
                    let _ = self.template_depth_stack.pop();
                    // Continue scanning template middle/tail
                    return self.scan_template_continuation();
                }
            }
        }

        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let start = self.pos as u32;
        let Some(b) = self.peek() else {
            return self.make_eof();
        };

        let token = match b {
            // Identifiers and keywords
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => self.scan_identifier_or_keyword(start),
            // Unicode identifier start (multi-byte UTF-8)
            0xC0..=0xFF => self.scan_identifier_or_keyword(start),
            // Numbers
            b'0'..=b'9' => self.scan_number(start),
            // Strings
            b'\'' | b'"' => self.scan_string(start),
            // Template literals
            b'`' => self.scan_template(start),
            // Punctuators and operators
            b'{' => {
                self.advance();
                self.brace_depth = self.brace_depth.saturating_add(1);
                self.make(TokenKind::LBrace, start)
            }
            b'}' => {
                self.advance();
                self.brace_depth = self.brace_depth.saturating_sub(1);
                self.make(TokenKind::RBrace, start)
            }
            b'(' => {
                self.advance();
                self.make(TokenKind::LParen, start)
            }
            b')' => {
                self.advance();
                self.make(TokenKind::RParen, start)
            }
            b'[' => {
                self.advance();
                self.make(TokenKind::LBracket, start)
            }
            b']' => {
                self.advance();
                self.make(TokenKind::RBracket, start)
            }
            b';' => {
                self.advance();
                self.make(TokenKind::Semicolon, start)
            }
            b',' => {
                self.advance();
                self.make(TokenKind::Comma, start)
            }
            b'~' => {
                self.advance();
                self.make(TokenKind::Tilde, start)
            }
            b'@' => {
                self.advance();
                self.make(TokenKind::At, start)
            }
            b'#' => {
                self.advance();
                self.scan_hash_or_private(start)
            }
            b':' => {
                self.advance();
                self.make(TokenKind::Colon, start)
            }
            b'.' => self.scan_dot(start),
            b'<' => self.scan_less_than(start),
            b'>' => self.scan_greater_than(start),
            b'=' => self.scan_equals(start),
            b'!' => self.scan_exclamation(start),
            b'+' => self.scan_plus(start),
            b'-' => self.scan_minus(start),
            b'*' => self.scan_star(start),
            b'/' => self.scan_slash(start),
            b'%' => self.scan_percent(start),
            b'&' => self.scan_ampersand(start),
            b'|' => self.scan_pipe(start),
            b'^' => self.scan_caret(start),
            b'?' => self.scan_question(start),
            _ => {
                // Skip unknown byte
                self.advance();
                self.errors.push(ParseError::new(
                    format!("unexpected character: {}", b as char),
                    start,
                    start.saturating_add(1),
                ));
                self.next_token()
            }
        };

        // Update expression-end tracking for regex disambiguation
        self.prev_token_is_expr_end = token_ends_expression(token.kind);

        token
    }

    /// Skip whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(b) = self.peek() {
                if is_whitespace(b) {
                    self.advance();
                } else {
                    break;
                }
            }

            // Check for comments
            if self.peek() == Some(b'/') {
                if self.peek_at(1) == Some(b'/') {
                    // Line comment
                    self.advance_by(2);
                    while let Some(b) = self.peek() {
                        if b == b'\n' || b == b'\r' {
                            break;
                        }
                        self.advance();
                    }
                    continue;
                } else if self.peek_at(1) == Some(b'*') {
                    // Block comment
                    self.advance_by(2);
                    loop {
                        match self.peek() {
                            None => {
                                #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                                self.errors.push(ParseError::new(
                                    "unterminated block comment",
                                    self.pos as u32,
                                    self.pos as u32,
                                ));
                                break;
                            }
                            Some(b'*') if self.peek_at(1) == Some(b'/') => {
                                self.advance_by(2);
                                break;
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                    continue;
                }
            }

            // Check for hashbang on first line
            if self.pos == 0 && self.peek() == Some(b'#') && self.peek_at(1) == Some(b'!') {
                while let Some(b) = self.peek() {
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            break;
        }
    }

    /// Make an EOF token.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    const fn make_eof(&self) -> Token {
        Token::new(
            TokenKind::Eof,
            self.source.len() as u32,
            self.source.len() as u32,
        )
    }

    /// Make a token ending at the current position.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    const fn make(&self, kind: TokenKind, start: u32) -> Token {
        Token::new(kind, start, self.pos as u32)
    }

    // --- Identifier / keyword scanning ---

    /// Scan an identifier or keyword.
    fn scan_identifier_or_keyword(&mut self, start: u32) -> Token {
        self.advance(); // consume first char
        while let Some(b) = self.peek() {
            if is_identifier_continue(b) {
                self.advance();
            } else {
                break;
            }
        }
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let text = self.source_text(start as usize, self.pos);
        let kind = keyword_from_str(text).unwrap_or(TokenKind::Identifier);
        self.make(kind, start)
    }

    /// Scan a JSX identifier (allows hyphens: `aria-hidden`, `data-testid`).
    fn scan_jsx_identifier(&mut self, start: u32) -> Token {
        self.advance(); // consume first char
        while let Some(b) = self.peek() {
            if is_identifier_continue(b) || b == b'-' {
                self.advance();
            } else {
                break;
            }
        }
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let text = self.source_text(start as usize, self.pos);
        // JSX identifiers with hyphens are always plain identifiers
        let kind = if text.contains('-') {
            TokenKind::Identifier
        } else {
            keyword_from_str(text).unwrap_or(TokenKind::Identifier)
        };
        self.make(kind, start)
    }

    /// After `#`, scan a private identifier.
    fn scan_hash_or_private(&mut self, start: u32) -> Token {
        // After `#`, if next is identifier start, it's a private name
        if let Some(b) = self.peek() {
            if is_identifier_start(b) {
                while let Some(b2) = self.peek() {
                    if is_identifier_continue(b2) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                return self.make(TokenKind::Hash, start);
            }
        }
        self.make(TokenKind::Hash, start)
    }

    // --- Number scanning ---

    /// Scan a numeric literal.
    #[allow(clippy::too_many_lines)]
    fn scan_number(&mut self, start: u32) -> Token {
        if self.peek() == Some(b'0') {
            self.advance();
            match self.peek() {
                Some(b'x' | b'X') => {
                    self.advance();
                    self.scan_hex_digits();
                }
                Some(b'o' | b'O') => {
                    self.advance();
                    self.scan_octal_digits();
                }
                Some(b'b' | b'B') => {
                    self.advance();
                    self.scan_binary_digits();
                }
                _ => {
                    // Decimal or legacy octal
                    self.scan_decimal_digits();
                    if self.peek() == Some(b'.') {
                        self.advance();
                        self.scan_decimal_digits();
                    }
                    self.scan_exponent();
                }
            }
        } else {
            self.scan_decimal_digits();
            if self.peek() == Some(b'.') {
                self.advance();
                self.scan_decimal_digits();
            }
            self.scan_exponent();
        }
        // BigInt suffix
        if self.peek() == Some(b'n') {
            self.advance();
        }
        self.make(TokenKind::Number, start)
    }

    /// Scan decimal digits (0-9), including underscores as separators.
    fn scan_decimal_digits(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Scan hex digits (0-9, a-f, A-F), including underscores.
    fn scan_hex_digits(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_hexdigit() || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Scan octal digits (0-7), including underscores.
    fn scan_octal_digits(&mut self) {
        while let Some(b) = self.peek() {
            if (b'0'..=b'7').contains(&b) || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Scan binary digits (0-1), including underscores.
    fn scan_binary_digits(&mut self) {
        while let Some(b) = self.peek() {
            if b == b'0' || b == b'1' || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Scan an optional exponent part (e/E followed by optional sign and digits).
    fn scan_exponent(&mut self) {
        if self.peek() == Some(b'e') || self.peek() == Some(b'E') {
            self.advance();
            if self.peek() == Some(b'+') || self.peek() == Some(b'-') {
                self.advance();
            }
            self.scan_decimal_digits();
        }
    }

    // --- String scanning ---

    /// Scan a string literal (single or double quoted).
    fn scan_string(&mut self, start: u32) -> Token {
        let quote = self.advance().unwrap_or(b'"');
        loop {
            match self.peek() {
                None | Some(b'\n' | b'\r') => {
                    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                    self.errors.push(ParseError::new(
                        "unterminated string literal",
                        start,
                        self.pos as u32,
                    ));
                    break;
                }
                Some(b'\\') => {
                    self.advance(); // backslash
                    self.advance(); // escaped character
                }
                Some(b) if b == quote => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.make(TokenKind::String, start)
    }

    // --- Template literal scanning ---

    /// Scan a template literal starting with `` ` ``.
    fn scan_template(&mut self, start: u32) -> Token {
        self.advance(); // consume backtick
        self.scan_template_content(start, true)
    }

    /// Scan template continuation after `}` in `${...}`.
    fn scan_template_continuation(&mut self) -> Token {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let start = self.pos as u32;
        self.advance(); // consume `}`
        self.brace_depth = self.brace_depth.saturating_sub(1);
        self.scan_template_content(start, false)
    }

    /// Scan template literal content until `` ` `` (end) or `${` (expression).
    fn scan_template_content(&mut self, start: u32, is_head: bool) -> Token {
        loop {
            match self.peek() {
                None => {
                    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                    self.errors.push(ParseError::new(
                        "unterminated template literal",
                        start,
                        self.pos as u32,
                    ));
                    let kind = if is_head {
                        TokenKind::NoSubstitutionTemplate
                    } else {
                        TokenKind::TemplateTail
                    };
                    return self.make(kind, start);
                }
                Some(b'\\') => {
                    self.advance(); // backslash
                    self.advance(); // escaped character
                }
                Some(b'`') => {
                    self.advance();
                    let kind = if is_head {
                        TokenKind::NoSubstitutionTemplate
                    } else {
                        TokenKind::TemplateTail
                    };
                    return self.make(kind, start);
                }
                Some(b'$') if self.peek_at(1) == Some(b'{') => {
                    self.advance_by(2);
                    // Save current brace depth + 1 so the closing `}` matches
                    // (the `}` check runs before the main match decrements brace_depth).
                    self.brace_depth = self.brace_depth.saturating_add(1);
                    self.template_depth_stack.push(self.brace_depth);
                    let kind = if is_head {
                        TokenKind::TemplateHead
                    } else {
                        TokenKind::TemplateMiddle
                    };
                    return self.make(kind, start);
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // --- Regex scanning ---

    /// Scan a regular expression literal.
    fn scan_regex(&mut self, start: u32) -> Token {
        self.advance(); // consume opening `/`
        let mut in_class = false;
        loop {
            match self.peek() {
                None | Some(b'\n' | b'\r') => {
                    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                    self.errors.push(ParseError::new(
                        "unterminated regular expression",
                        start,
                        self.pos as u32,
                    ));
                    break;
                }
                Some(b'\\') => {
                    self.advance();
                    self.advance(); // skip escaped char
                }
                Some(b'[') if !in_class => {
                    in_class = true;
                    self.advance();
                }
                Some(b']') if in_class => {
                    in_class = false;
                    self.advance();
                }
                Some(b'/') if !in_class => {
                    self.advance(); // closing `/`
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
        // Scan flags
        while let Some(b) = self.peek() {
            if b.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }
        self.make(TokenKind::RegExp, start)
    }

    // --- Multi-character operator scanning ---

    /// Scan `.`, `...`, or a number starting with `.`.
    fn scan_dot(&mut self, start: u32) -> Token {
        self.advance();
        if self.peek() == Some(b'.') && self.peek_at(1) == Some(b'.') {
            self.advance_by(2);
            return self.make(TokenKind::DotDotDot, start);
        }
        // Check for `.123` decimal literal
        if let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                self.scan_decimal_digits();
                self.scan_exponent();
                return self.make(TokenKind::Number, start);
            }
        }
        self.make(TokenKind::Dot, start)
    }

    /// Scan `<`, `<=`, `<<`, `<<=`.
    fn scan_less_than(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::LessEq, start)
            }
            Some(b'<') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::LessLessEq, start)
                } else {
                    self.make(TokenKind::LessLess, start)
                }
            }
            _ => self.make(TokenKind::LAngle, start),
        }
    }

    /// Scan `>`, `>=`, `>>`, `>>=`, `>>>`, `>>>=`.
    fn scan_greater_than(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::GreaterEq, start)
            }
            Some(b'>') => {
                self.advance();
                match self.peek() {
                    Some(b'=') => {
                        self.advance();
                        self.make(TokenKind::GreaterGreaterEq, start)
                    }
                    Some(b'>') => {
                        self.advance();
                        if self.peek() == Some(b'=') {
                            self.advance();
                            self.make(TokenKind::GreaterGreaterGreaterEq, start)
                        } else {
                            self.make(TokenKind::GreaterGreaterGreater, start)
                        }
                    }
                    _ => self.make(TokenKind::GreaterGreater, start),
                }
            }
            _ => self.make(TokenKind::RAngle, start),
        }
    }

    /// Scan `=`, `==`, `===`, `=>`.
    fn scan_equals(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'=') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::EqEqEq, start)
                } else {
                    self.make(TokenKind::EqEq, start)
                }
            }
            Some(b'>') => {
                self.advance();
                self.make(TokenKind::Arrow, start)
            }
            _ => self.make(TokenKind::Eq, start),
        }
    }

    /// Scan `!`, `!=`, `!==`.
    fn scan_exclamation(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'=') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::NotEqEq, start)
                } else {
                    self.make(TokenKind::NotEq, start)
                }
            }
            _ => self.make(TokenKind::Bang, start),
        }
    }

    /// Scan `+`, `++`, `+=`.
    fn scan_plus(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'+') => {
                self.advance();
                self.make(TokenKind::PlusPlus, start)
            }
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::PlusEq, start)
            }
            _ => self.make(TokenKind::Plus, start),
        }
    }

    /// Scan `-`, `--`, `-=`.
    fn scan_minus(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'-') => {
                self.advance();
                self.make(TokenKind::MinusMinus, start)
            }
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::MinusEq, start)
            }
            _ => self.make(TokenKind::Minus, start),
        }
    }

    /// Scan `*`, `**`, `*=`, `**=`.
    fn scan_star(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'*') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::StarStarEq, start)
                } else {
                    self.make(TokenKind::StarStar, start)
                }
            }
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::StarEq, start)
            }
            _ => self.make(TokenKind::Star, start),
        }
    }

    /// Scan `/`, `/=`, `//` (comment), `/*` (comment), or regex.
    fn scan_slash(&mut self, start: u32) -> Token {
        // Comments are already handled in skip_whitespace_and_comments.
        // At this point, `/` is either division or regex.
        if !self.prev_token_is_expr_end {
            // Regex context: the previous token was not an expression end
            return self.scan_regex(start);
        }
        self.advance();
        if self.peek() == Some(b'=') {
            self.advance();
            self.make(TokenKind::SlashEq, start)
        } else {
            self.make(TokenKind::Slash, start)
        }
    }

    /// Scan `%`, `%=`.
    fn scan_percent(&mut self, start: u32) -> Token {
        self.advance();
        if self.peek() == Some(b'=') {
            self.advance();
            self.make(TokenKind::PercentEq, start)
        } else {
            self.make(TokenKind::Percent, start)
        }
    }

    /// Scan `&`, `&&`, `&=`, `&&=`.
    fn scan_ampersand(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'&') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::AmpAmpEq, start)
                } else {
                    self.make(TokenKind::AmpAmp, start)
                }
            }
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::AmpEq, start)
            }
            _ => self.make(TokenKind::Amp, start),
        }
    }

    /// Scan `|`, `||`, `|=`, `||=`.
    fn scan_pipe(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'|') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::PipePipeEq, start)
                } else {
                    self.make(TokenKind::PipePipe, start)
                }
            }
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::PipeEq, start)
            }
            _ => self.make(TokenKind::Pipe, start),
        }
    }

    /// Scan `^`, `^=`.
    fn scan_caret(&mut self, start: u32) -> Token {
        self.advance();
        if self.peek() == Some(b'=') {
            self.advance();
            self.make(TokenKind::CaretEq, start)
        } else {
            self.make(TokenKind::Caret, start)
        }
    }

    /// Scan `?`, `?.`, `??`, `??=`.
    fn scan_question(&mut self, start: u32) -> Token {
        self.advance();
        match self.peek() {
            Some(b'.') => {
                // `?.` but not `?.digit` (that would be `?` followed by `.5`)
                if let Some(b) = self.peek_at(1) {
                    if b.is_ascii_digit() {
                        return self.make(TokenKind::Question, start);
                    }
                }
                self.advance();
                self.make(TokenKind::QuestionDot, start)
            }
            Some(b'?') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.make(TokenKind::QuestionQuestionEq, start)
                } else {
                    self.make(TokenKind::QuestionQuestion, start)
                }
            }
            _ => self.make(TokenKind::Question, start),
        }
    }

    // --- JSX scanning ---

    /// Scan a token in JSX child context (text between tags).
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn scan_jsx_child(&mut self) -> Token {
        if self.pos >= self.source.len() {
            return self.make_eof();
        }
        let start = self.pos as u32;
        match self.peek() {
            Some(b'{') => {
                self.advance();
                self.brace_depth = self.brace_depth.saturating_add(1);
                self.make(TokenKind::LBrace, start)
            }
            Some(b'<') => {
                self.advance();
                self.make(TokenKind::LAngle, start)
            }
            _ => {
                // Scan JSX text
                while let Some(b) = self.peek() {
                    if b == b'{' || b == b'<' {
                        break;
                    }
                    self.advance();
                }
                self.make(TokenKind::JsxText, start)
            }
        }
    }

    /// Scan a token inside a JSX opening/closing tag.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn scan_jsx_tag_token(&mut self) -> Token {
        let start = self.pos as u32;
        match self.peek() {
            Some(b'>') => {
                self.advance();
                self.make(TokenKind::RAngle, start)
            }
            Some(b'/') => {
                self.advance();
                self.make(TokenKind::Slash, start)
            }
            Some(b'=') => {
                self.advance();
                self.make(TokenKind::Eq, start)
            }
            Some(b'{') => {
                self.advance();
                self.brace_depth = self.brace_depth.saturating_add(1);
                self.make(TokenKind::LBrace, start)
            }
            Some(b'\'' | b'"') => self.scan_string(start),
            Some(b'<') => {
                self.advance();
                self.make(TokenKind::LAngle, start)
            }
            Some(b'.') => {
                self.advance();
                self.make(TokenKind::Dot, start)
            }
            Some(b':') => {
                self.advance();
                self.make(TokenKind::Colon, start)
            }
            Some(b) if is_identifier_start(b) => self.scan_jsx_identifier(start),
            _ => {
                self.advance();
                self.errors.push(ParseError::new(
                    "unexpected character in JSX tag",
                    start,
                    start.saturating_add(1),
                ));
                self.next_token()
            }
        }
    }
}

// --- Character classification helpers ---

/// Whether a byte is whitespace (space, tab, newline, carriage return, etc.).
#[must_use]
const fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C | 0xA0)
}

/// Whether a byte can start an identifier (letter, underscore, dollar, or multi-byte UTF-8 start).
#[must_use]
const fn is_identifier_start(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' | 0xC0..=0xFF)
}

/// Whether a byte can continue an identifier.
#[must_use]
const fn is_identifier_continue(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' | 0x80..=0xFF)
}

/// Whether a token kind ends an expression (for regex vs division disambiguation).
///
/// If `true`, a subsequent `/` is treated as division. If `false`, it's a regex.
#[must_use]
const fn token_ends_expression(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::Number
            | TokenKind::String
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::This
            | TokenKind::Super
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
            | TokenKind::NoSubstitutionTemplate
            | TokenKind::TemplateTail
            | TokenKind::RegExp
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus
    )
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::token::TokenKind;

    /// Lex all tokens from source, excluding EOF.
    fn lex_all(source: &str) -> Vec<(TokenKind, &str)> {
        let mut lexer = Lexer::new(source.as_bytes());
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            let start = tok.start as usize;
            let end = tok.end as usize;
            let text = &source[start..end];
            tokens.push((tok.kind, text));
        }
        tokens
    }

    #[test]
    fn simple_identifiers() {
        let tokens = lex_all("foo bar baz");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], (TokenKind::Identifier, "foo"));
        assert_eq!(tokens[1], (TokenKind::Identifier, "bar"));
        assert_eq!(tokens[2], (TokenKind::Identifier, "baz"));
    }

    #[test]
    fn keywords() {
        let tokens = lex_all("const let var if else function return");
        assert_eq!(tokens[0].0, TokenKind::Const);
        assert_eq!(tokens[1].0, TokenKind::Let);
        assert_eq!(tokens[2].0, TokenKind::Var);
        assert_eq!(tokens[3].0, TokenKind::If);
        assert_eq!(tokens[4].0, TokenKind::Else);
        assert_eq!(tokens[5].0, TokenKind::Function);
        assert_eq!(tokens[6].0, TokenKind::Return);
    }

    #[test]
    fn numbers() {
        let tokens = lex_all("42 3.14 0xFF 0o77 0b1010 100n 1_000");
        assert!(tokens.iter().all(|(k, _)| *k == TokenKind::Number));
        assert_eq!(tokens.len(), 7);
    }

    #[test]
    fn strings() {
        let tokens = lex_all(r#""hello" 'world' "esc\"ape""#);
        assert!(tokens.iter().all(|(k, _)| *k == TokenKind::String));
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn template_no_substitution() {
        let tokens = lex_all("`hello world`");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, TokenKind::NoSubstitutionTemplate);
    }

    #[test]
    fn template_with_substitution() {
        let tokens = lex_all("`hello ${name}!`");
        assert_eq!(tokens[0].0, TokenKind::TemplateHead);
        assert_eq!(tokens[1].0, TokenKind::Identifier);
        assert_eq!(tokens[2].0, TokenKind::TemplateTail);
    }

    #[test]
    fn basic_operators() {
        let tokens = lex_all("+ - * ** = == ===");
        let kinds: Vec<_> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::StarStar,
                TokenKind::Eq,
                TokenKind::EqEq,
                TokenKind::EqEqEq,
            ]
        );
    }

    #[test]
    fn punctuators() {
        let tokens = lex_all("{ } ( ) [ ] ; , . ... : ?");
        let kinds: Vec<_> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::DotDotDot,
                TokenKind::Colon,
                TokenKind::Question,
            ]
        );
    }

    #[test]
    fn line_comments() {
        let tokens = lex_all("foo // comment\nbar");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].1, "foo");
        assert_eq!(tokens[1].1, "bar");
    }

    #[test]
    fn block_comments() {
        let tokens = lex_all("foo /* block */ bar");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].1, "foo");
        assert_eq!(tokens[1].1, "bar");
    }

    #[test]
    fn regex_after_assignment() {
        // `= /pattern/g` — regex after `=`
        let tokens = lex_all("x = /pattern/g");
        assert_eq!(tokens[0].0, TokenKind::Identifier);
        assert_eq!(tokens[1].0, TokenKind::Eq);
        assert_eq!(tokens[2].0, TokenKind::RegExp);
    }

    #[test]
    fn division_after_identifier() {
        // `x / y` — division after identifier
        let tokens = lex_all("x / y");
        assert_eq!(tokens[0].0, TokenKind::Identifier);
        assert_eq!(tokens[1].0, TokenKind::Slash);
        assert_eq!(tokens[2].0, TokenKind::Identifier);
    }

    #[test]
    fn arrow_function() {
        let tokens = lex_all("(x) => x");
        let kinds: Vec<_> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LParen,
                TokenKind::Identifier,
                TokenKind::RParen,
                TokenKind::Arrow,
                TokenKind::Identifier,
            ]
        );
    }

    #[test]
    fn optional_chaining() {
        let tokens = lex_all("a?.b ?? c");
        let kinds: Vec<_> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::QuestionDot,
                TokenKind::Identifier,
                TokenKind::QuestionQuestion,
                TokenKind::Identifier,
            ]
        );
    }

    #[test]
    fn hashbang() {
        let tokens = lex_all("#!/usr/bin/env node\nconsole");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].1, "console");
    }

    #[test]
    fn spread_operator() {
        let tokens = lex_all("...args");
        assert_eq!(tokens[0].0, TokenKind::DotDotDot);
        assert_eq!(tokens[1].0, TokenKind::Identifier);
    }
}
