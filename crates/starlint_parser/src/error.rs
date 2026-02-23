//! Parse error types and diagnostics.

use miette::Diagnostic;
use thiserror::Error;

/// A parse error with source location.
#[derive(Debug, Clone, Error, Diagnostic)]
#[error("{message}")]
pub struct ParseError {
    /// Human-readable error message.
    pub message: String,
    /// Byte offset where the error starts.
    pub start: u32,
    /// Byte offset where the error ends.
    pub end: u32,
}

impl ParseError {
    /// Create a new parse error.
    #[must_use]
    pub fn new(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            message: message.into(),
            start,
            end,
        }
    }
}
