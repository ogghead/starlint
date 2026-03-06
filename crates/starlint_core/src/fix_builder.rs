//! Ergonomic builder for constructing [`Fix`] objects.
//!
//! Provides a fluent API to compose multi-edit fixes from individual edits
//! or from utility functions in [`fix_utils`](super::fix_utils).

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Span};
use starlint_plugin_sdk::rule::FixKind;

/// Builder for constructing a [`Fix`] with one or more edits.
///
/// # Example
///
/// ```ignore
/// let fix = FixBuilder::new("Replace `let` with `const`", FixKind::SafeFix)
///     .replace(let_span, "const")
///     .build();
/// ```
pub struct FixBuilder {
    /// Safety classification of the fix.
    kind: FixKind,
    /// Human-readable description of the fix.
    message: String,
    /// Accumulated edits.
    edits: Vec<Edit>,
}

impl FixBuilder {
    /// Create a new builder with the given fix description and safety classification.
    #[must_use]
    pub fn new(message: impl Into<String>, kind: FixKind) -> Self {
        Self {
            kind,
            message: message.into(),
            edits: Vec::new(),
        }
    }

    /// Add a replacement edit: replace `span` with `replacement`.
    #[must_use]
    pub fn replace(mut self, span: Span, replacement: impl Into<String>) -> Self {
        self.edits.push(Edit {
            span,
            replacement: replacement.into(),
        });
        self
    }

    /// Add a deletion edit: remove content at `span`.
    #[must_use]
    pub fn delete(mut self, span: Span) -> Self {
        self.edits.push(Edit {
            span,
            replacement: String::new(),
        });
        self
    }

    /// Add an insertion edit: insert `text` at `offset`.
    #[must_use]
    pub fn insert_at(mut self, offset: u32, text: impl Into<String>) -> Self {
        self.edits.push(Edit {
            span: Span::new(offset, offset),
            replacement: text.into(),
        });
        self
    }

    /// Add a pre-built edit (from [`fix_utils`](super::fix_utils) or manual construction).
    #[must_use]
    pub fn edit(mut self, edit: Edit) -> Self {
        self.edits.push(edit);
        self
    }

    /// Add multiple pre-built edits.
    #[must_use]
    pub fn edits(mut self, edits: Vec<Edit>) -> Self {
        self.edits.extend(edits);
        self
    }

    /// Build the [`Fix`]. Returns `None` if no edits were added.
    #[must_use]
    pub fn build(self) -> Option<Fix> {
        if self.edits.is_empty() {
            return None;
        }
        Some(Fix {
            kind: self.kind,
            message: self.message,
            edits: self.edits,
            is_snippet: false,
        })
    }

    /// Build a snippet [`Fix`] whose replacements contain LSP snippet syntax
    /// (`$1`, `${1:placeholder}`).
    ///
    /// Snippet fixes are only applied by editors that support `SnippetTextEdit`.
    /// The CLI always skips them.
    #[must_use]
    pub fn build_snippet(self) -> Option<Fix> {
        if self.edits.is_empty() {
            return None;
        }
        Some(Fix {
            kind: self.kind,
            message: self.message,
            edits: self.edits,
            is_snippet: true,
        })
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_single_replace() {
        let result = FixBuilder::new("test", FixKind::SafeFix)
            .replace(Span::new(0, 3), "const")
            .build();
        assert!(result.is_some(), "should produce a fix");
        assert_eq!(
            result.as_ref().map(|f| f.edits.len()),
            Some(1),
            "should have one edit"
        );
        assert_eq!(
            result.as_ref().map(|f| f.message.as_str()),
            Some("test"),
            "message should match"
        );
        assert_eq!(
            result.and_then(|f| f.edits.first().map(|e| e.replacement.clone())),
            Some("const".to_owned()),
            "replacement should match"
        );
    }

    #[test]
    fn test_multi_edit() {
        let result = FixBuilder::new("multi", FixKind::SafeFix)
            .replace(Span::new(0, 3), "const")
            .delete(Span::new(10, 20))
            .insert_at(25, "// comment")
            .build();
        assert_eq!(
            result.as_ref().map(|f| f.edits.len()),
            Some(3),
            "should have three edits"
        );
    }

    #[test]
    fn test_no_edits_returns_none() {
        let result = FixBuilder::new("empty", FixKind::SafeFix).build();
        assert!(result.is_none(), "no edits should return None");
    }

    #[test]
    fn test_edit_from_vec() {
        let edits = vec![Edit {
            span: Span::new(0, 1),
            replacement: "x".to_owned(),
        }];
        let result = FixBuilder::new("from vec", FixKind::SafeFix)
            .edits(edits)
            .build();
        assert!(result.is_some(), "should produce a fix from vec");
    }

    #[test]
    fn test_single_edit() {
        let edit = Edit {
            span: Span::new(5, 10),
            replacement: "hello".to_owned(),
        };
        let result = FixBuilder::new("single", FixKind::SafeFix)
            .edit(edit)
            .build();
        assert_eq!(
            result.as_ref().map(|f| f.edits.len()),
            Some(1),
            "should have one edit"
        );
    }

    #[test]
    fn test_delete_creates_empty_replacement() {
        let result = FixBuilder::new("del", FixKind::SafeFix)
            .delete(Span::new(0, 5))
            .build();
        assert_eq!(
            result.and_then(|f| f.edits.first().map(|e| e.replacement.clone())),
            Some(String::new()),
            "delete should produce empty replacement"
        );
    }

    #[test]
    fn test_insert_at_creates_zero_width_span() {
        let result = FixBuilder::new("ins", FixKind::SafeFix)
            .insert_at(10, "text")
            .build();
        assert!(result.is_some(), "should produce a fix");
        let edit_span_start = result
            .as_ref()
            .and_then(|f| f.edits.first().map(|e| e.span.start));
        let edit_span_end = result
            .as_ref()
            .and_then(|f| f.edits.first().map(|e| e.span.end));
        assert_eq!(edit_span_start, Some(10), "start should be offset");
        assert_eq!(
            edit_span_end,
            Some(10),
            "end should equal start for insertion"
        );
        assert_eq!(
            result.and_then(|f| f.edits.first().map(|e| e.replacement.clone())),
            Some("text".to_owned()),
            "replacement should be text"
        );
    }
}
