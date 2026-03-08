//! Hand-written JS/TS/JSX parser producing [`starlint_ast::AstTree`] directly.
//!
//! The parser constructs the flat indexed AST during parsing with no
//! intermediate representation, eliminating the copy overhead of the
//! oxc-to-`AstTree` conversion step.

// Parser-specific clippy relaxations: recursive descent parsers necessarily
// use indexing (source bytes), `as` casts (byte offsets), discarded results
// from `expect()`, and variable shadowing (`id` for different node IDs).
#![allow(
    clippy::let_underscore_must_use,
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::shadow_unrelated,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::match_same_arms,
    clippy::multiple_inherent_impl
)]

pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

#[cfg(test)]
mod proptest_parser;

use error::ParseError;
use starlint_ast::AstTree;

/// Source type configuration for the parser.
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Enable JSX syntax.
    pub jsx: bool,
    /// Enable TypeScript syntax.
    pub typescript: bool,
    /// Whether to parse as a module (allows `import`/`export`).
    pub module: bool,
}

impl ParseOptions {
    /// Infer parse options from a file path.
    #[must_use]
    pub fn from_path(path: &std::path::Path) -> Self {
        let ext = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");
        match ext {
            "tsx" => Self {
                jsx: true,
                typescript: true,
                module: true,
            },
            "ts" | "mts" | "cts" => Self {
                jsx: false,
                typescript: true,
                module: true,
            },
            "jsx" | "mjsx" => Self {
                jsx: true,
                typescript: false,
                module: true,
            },
            "mjs" | "mts2" => Self {
                jsx: false,
                typescript: false,
                module: true,
            },
            "cjs" | "cts2" => Self {
                jsx: false,
                typescript: false,
                module: false,
            },
            _ => Self {
                // .js defaults: JSX enabled, module mode
                jsx: true,
                typescript: false,
                module: true,
            },
        }
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            jsx: true,
            typescript: false,
            module: true,
        }
    }
}

/// Result of parsing a source file.
pub struct ParseResult {
    /// The constructed AST.
    pub tree: AstTree,
    /// Parse errors encountered.
    pub errors: Vec<ParseError>,
    /// Whether the parser panicked and error recovery was used.
    pub panicked: bool,
}

/// Parse source text into an [`AstTree`].
#[must_use]
pub fn parse(source: &str, options: ParseOptions) -> ParseResult {
    let mut parser = parser::Parser::new(source, options);
    parser.parse()
}
