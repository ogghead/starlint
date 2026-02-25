//! Open document state tracking for the LSP server.
//!
//! Stores the current text and version for each open document, plus
//! cached fix data for code actions.

use tower_lsp::lsp_types;

/// A code action derived from a starlint diagnostic fix, ready to return to the client.
#[derive(Debug, Clone)]
pub struct CachedFix {
    /// The diagnostic range this fix applies to.
    pub diagnostic_range: lsp_types::Range,
    /// The code action to return to the client.
    pub action: lsp_types::CodeAction,
}

/// State for a single open document.
#[derive(Debug)]
pub struct DocumentState {
    /// Document version (monotonically increasing).
    pub version: i32,
    /// Current full text of the document.
    pub text: String,
    /// Cached code actions from the most recent lint pass.
    pub cached_fixes: Vec<CachedFix>,
}

impl DocumentState {
    /// Create a new document state.
    #[must_use]
    pub const fn new(version: i32, text: String) -> Self {
        Self {
            version,
            text,
            cached_fixes: Vec::new(),
        }
    }

    /// Update the document text and version.
    pub fn update(&mut self, version: i32, text: String) {
        self.version = version;
        self.text = text;
        self.cached_fixes.clear();
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = DocumentState::new(1, "hello".to_owned());
        assert_eq!(doc.version, 1, "initial version");
        assert_eq!(doc.text, "hello", "initial text");
        assert!(doc.cached_fixes.is_empty(), "no cached fixes initially");
    }

    #[test]
    fn test_document_update_clears_fixes() {
        let mut doc = DocumentState::new(1, "hello".to_owned());
        doc.cached_fixes.push(CachedFix {
            diagnostic_range: lsp_types::Range::default(),
            action: lsp_types::CodeAction::default(),
        });
        assert_eq!(doc.cached_fixes.len(), 1, "should have one cached fix");

        doc.update(2, "world".to_owned());
        assert_eq!(doc.version, 2, "updated version");
        assert_eq!(doc.text, "world", "updated text");
        assert!(
            doc.cached_fixes.is_empty(),
            "cached fixes cleared on update"
        );
    }
}
