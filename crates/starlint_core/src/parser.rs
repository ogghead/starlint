//! oxc parser wrapper.
//!
//! Wraps the oxc parser to provide a clean interface for parsing JS/TS files.

use std::path::Path;

use miette::miette;
use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

/// Result of parsing a single file.
pub struct ParseResult<'a> {
    /// The parsed AST.
    pub program: oxc_ast::ast::Program<'a>,
    /// Parse errors (non-fatal; the AST may still be partially valid).
    pub panicked: bool,
}

/// Parse a source file into an oxc AST.
///
/// The allocator must outlive the returned AST. The file path is used
/// to determine the source type (JS, TS, JSX, TSX).
pub fn parse_file<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    file_path: &Path,
) -> miette::Result<ParseResult<'a>> {
    let source_type = SourceType::from_path(file_path)
        .map_err(|_err| miette!("unsupported file type: {}", file_path.display()))?;

    let ret = Parser::new(allocator, source_text, source_type)
        .with_options(ParseOptions::default())
        .parse();

    Ok(ParseResult {
        program: ret.program,
        panicked: ret.panicked,
    })
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_parse_valid_js() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        assert!(result.is_ok(), "valid JS should parse successfully");
    }

    #[test]
    fn test_parse_valid_ts() {
        let allocator = Allocator::default();
        let source = "const x: number = 1;";
        let result = parse_file(&allocator, source, Path::new("test.ts"));
        assert!(result.is_ok(), "valid TS should parse successfully");
    }

    #[test]
    fn test_parse_valid_tsx() {
        let allocator = Allocator::default();
        let source = "const App = () => <div />;";
        let result = parse_file(&allocator, source, Path::new("test.tsx"));
        assert!(result.is_ok(), "valid TSX should parse successfully");
    }

    #[test]
    fn test_parse_unsupported_extension() {
        let allocator = Allocator::default();
        let result = parse_file(&allocator, "", Path::new("test.py"));
        assert!(result.is_err(), "unsupported extension should fail");
    }
}
