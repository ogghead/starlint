//! Core primitive types used throughout the AST.

use serde::{Deserialize, Serialize};

/// Index into an [`AstTree`](crate::AstTree)'s node arena.
///
/// Nodes reference their children by `NodeId`, forming a flat indexed tree
/// that avoids recursive type definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    /// The root node is always at index 0.
    pub const ROOT: Self = Self(0);

    /// Sentinel value meaning "no node". Used in parent arrays for the root.
    pub const NONE: Self = Self(u32::MAX);

    /// Return the raw index as `usize`.
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for NodeId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<NodeId> for u32 {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

/// A byte-offset span in source text (start inclusive, end exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

impl Span {
    /// Create a new span.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// An empty/dummy span at offset 0.
    pub const EMPTY: Self = Self { start: 0, end: 0 };

    /// Byte length of this span (saturating subtraction to avoid overflow).
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether this span is empty (zero-length).
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Return the source text slice for this span.
    #[must_use]
    pub fn source_text<'a>(&self, source: &'a str) -> Option<&'a str> {
        let start = usize::try_from(self.start).ok()?;
        let end = usize::try_from(self.end).ok()?;
        source.get(start..end)
    }
}

impl From<Span> for starlint_plugin_sdk::diagnostic::Span {
    fn from(span: Span) -> Self {
        Self::new(span.start, span.end)
    }
}

impl From<starlint_plugin_sdk::diagnostic::Span> for Span {
    fn from(span: starlint_plugin_sdk::diagnostic::Span) -> Self {
        Self::new(span.start, span.end)
    }
}

#[cfg(test)]
mod tests {
    use super::{NodeId, Span};

    #[test]
    fn node_id_root() {
        assert_eq!(NodeId::ROOT.index(), 0, "root should be index 0");
    }

    #[test]
    fn node_id_roundtrip() {
        let id = NodeId(42);
        let raw: u32 = id.into();
        let back: NodeId = raw.into();
        assert_eq!(id, back, "roundtrip should preserve value");
    }

    #[test]
    fn span_basics() {
        let s = Span::new(10, 20);
        assert_eq!(s.len(), 10, "len should be 10");
        assert!(!s.is_empty(), "non-empty span");
        assert!(Span::EMPTY.is_empty(), "empty span");
    }

    #[test]
    fn span_source_text() {
        let source = "hello world";
        let s = Span::new(6, 11);
        assert_eq!(
            s.source_text(source),
            Some("world"),
            "should extract 'world'"
        );
    }

    #[test]
    fn span_from_sdk_span() {
        let sdk_span = starlint_plugin_sdk::diagnostic::Span::new(10, 20);
        let ast_span: Span = sdk_span.into();
        assert_eq!(ast_span.start, 10, "start should be preserved");
        assert_eq!(ast_span.end, 20, "end should be preserved");
    }

    #[test]
    fn span_into_sdk_span() {
        let ast_span = Span::new(5, 15);
        let sdk_span: starlint_plugin_sdk::diagnostic::Span = ast_span.into();
        assert_eq!(sdk_span.start, 5, "start should be preserved");
        assert_eq!(sdk_span.end, 15, "end should be preserved");
    }
}
