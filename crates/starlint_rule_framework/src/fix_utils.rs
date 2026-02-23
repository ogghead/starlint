//! Fix utility functions for common autofix patterns.
//!
//! Provides composable helpers that return [`Edit`] or `Vec<Edit>`,
//! allowing rules to assemble `Fix` objects from building blocks.

use starlint_plugin_sdk::diagnostic::{Edit, Span};

/// Safely extract source text for a span.
///
/// Returns `None` if the span is out of bounds or lands on a non-char-boundary.
#[must_use]
pub fn source_text_for_span(source: &str, span: Span) -> Option<&str> {
    let start = usize::try_from(span.start).ok()?;
    let end = usize::try_from(span.end).ok()?;
    if start > end
        || end > source.len()
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
    {
        return None;
    }
    source.get(start..end)
}

/// Create an edit that replaces content at the given span.
#[must_use]
pub fn replace(span: Span, replacement: impl Into<String>) -> Edit {
    Edit {
        span,
        replacement: replacement.into(),
    }
}

/// Create an edit that deletes content at the given span.
#[must_use]
pub const fn delete(span: Span) -> Edit {
    Edit {
        span,
        replacement: String::new(),
    }
}

/// Create an edit that inserts text before the given byte offset.
#[must_use]
pub fn insert_before(offset: u32, text: impl Into<String>) -> Edit {
    Edit {
        span: Span::new(offset, offset),
        replacement: text.into(),
    }
}

/// Delete a statement including its leading indentation and trailing newline.
///
/// Expands backward from `span.start` to consume leading whitespace (spaces/tabs)
/// on the same line, and forward from `span.end` to consume a trailing newline.
/// Falls back to bare span deletion if the statement shares a line with other code.
#[must_use]
pub fn delete_statement(source: &str, span: Span) -> Edit {
    let start = usize::try_from(span.start).unwrap_or(0);
    let end = usize::try_from(span.end).unwrap_or(0);

    if start > source.len() || end > source.len() || start > end {
        return delete(span);
    }

    // Walk backward to consume leading whitespace (spaces/tabs) on the same line.
    let mut expanded_start = start;
    while expanded_start > 0 {
        let prev = expanded_start.saturating_sub(1);
        match source.as_bytes().get(prev) {
            Some(b' ' | b'\t') => expanded_start = prev,
            // If we reach a newline, include it (we're deleting the whole line).
            Some(b'\n') => {
                expanded_start = prev;
                break;
            }
            // Hit non-whitespace before start of line — statement shares a line.
            // Don't expand backward.
            _ => {
                expanded_start = start;
                break;
            }
        }
    }

    // Walk forward to consume trailing whitespace and newline.
    let mut expanded_end = end;
    while expanded_end < source.len() {
        match source.as_bytes().get(expanded_end) {
            Some(b' ' | b'\t') => expanded_end = expanded_end.saturating_add(1),
            Some(b'\n') => {
                expanded_end = expanded_end.saturating_add(1);
                break;
            }
            Some(b'\r') => {
                expanded_end = expanded_end.saturating_add(1);
                // Also consume \n in \r\n
                if source.as_bytes().get(expanded_end) == Some(&b'\n') {
                    expanded_end = expanded_end.saturating_add(1);
                }
                break;
            }
            _ => break,
        }
    }

    let new_start = u32::try_from(expanded_start).unwrap_or(span.start);
    let new_end = u32::try_from(expanded_end).unwrap_or(span.end);

    Edit {
        span: Span::new(new_start, new_end),
        replacement: String::new(),
    }
}

/// Remove a JSX attribute and its surrounding whitespace.
///
/// Walks backward from `attr_span.start` to consume preceding whitespace
/// (spaces/tabs), producing a clean removal like `<div foo bar>` → `<div bar>`.
#[must_use]
pub fn remove_jsx_attr(source: &str, attr_span: Span) -> Edit {
    let start = usize::try_from(attr_span.start).unwrap_or(0);

    if start > source.len() {
        return delete(attr_span);
    }

    // Walk backward to consume preceding whitespace (space/tab).
    let mut expanded_start = start;
    while expanded_start > 0 {
        let prev = expanded_start.saturating_sub(1);
        match source.as_bytes().get(prev) {
            Some(b' ' | b'\t') => expanded_start = prev,
            _ => break,
        }
    }

    let new_start = u32::try_from(expanded_start).unwrap_or(attr_span.start);

    Edit {
        span: Span::new(new_start, attr_span.end),
        replacement: String::new(),
    }
}

// ── JSX utilities ───────────────────────────────────────────────────

/// Find the byte offset just before the `>` or `/>` of a JSX opening element.
///
/// Returns a suitable position for inserting a new attribute (e.g. ` alt="..."`).
/// Falls back to `opening_span.end - 1` if the closing delimiter can't be found.
#[must_use]
pub fn jsx_attr_insert_offset(source: &str, opening_span: Span) -> u32 {
    let end = usize::try_from(opening_span.end).unwrap_or(0);
    if end == 0 || end > source.len() {
        return opening_span.end.saturating_sub(1);
    }

    // Walk backward from span end to find `>` or `/>`.
    let mut pos = end;
    while pos > 0 {
        pos = pos.saturating_sub(1);
        if source.as_bytes().get(pos) == Some(&b'>') {
            // Check for `/>`
            if pos > 0 && source.as_bytes().get(pos.saturating_sub(1)) == Some(&b'/') {
                // Skip whitespace before />
                let mut insert = pos.saturating_sub(1);
                while insert > 0
                    && matches!(
                        source.as_bytes().get(insert.saturating_sub(1)),
                        Some(b' ' | b'\t' | b'\n' | b'\r')
                    )
                {
                    insert = insert.saturating_sub(1);
                }
                return u32::try_from(insert)
                    .unwrap_or_else(|_| opening_span.end.saturating_sub(2));
            }
            // Plain `>` — insert before it.
            return u32::try_from(pos).unwrap_or_else(|_| opening_span.end.saturating_sub(1));
        }
    }

    opening_span.end.saturating_sub(1)
}

// ── Semantic utilities ───────────────────────────────────────────────

/// Generate edits to rename a symbol (declaration + all references).
///
/// Returns one edit for the declaration site (`decl_span`) and one edit per
/// resolved reference. The caller must verify that renaming is safe (no
/// conflicts in target scopes).
#[must_use]
pub fn rename_symbol_edits(
    scope_data: &starlint_scope::ScopeData,
    symbol_id: starlint_scope::SymbolId,
    new_name: &str,
    decl_span: Span,
) -> Vec<Edit> {
    scope_data.rename_symbol_edits(symbol_id, new_name, decl_span)
}

// ── Import utilities ─────────────────────────────────────────────────

/// Generate edits to merge two named import declarations from the same source.
///
/// Given:
///   `import { foo } from 'mod';`
///   `import { bar } from 'mod';`
/// Produces edits that result in:
///   `import { foo, bar } from 'mod';`
///
/// Returns an empty vec if either import is not a simple named import
/// or the specifiers cannot be extracted.
#[must_use]
pub fn merge_import_edits(source: &str, first_span: Span, second_span: Span) -> Vec<Edit> {
    let Some(first_text) = source_text_for_span(source, first_span) else {
        return Vec::new();
    };
    let Some(second_text) = source_text_for_span(source, second_span) else {
        return Vec::new();
    };

    // Verify both imports have brace specifiers.
    if extract_brace_specifiers(first_text).is_none() {
        return Vec::new();
    }
    let Some(second_specs) = extract_brace_specifiers(second_text) else {
        return Vec::new();
    };

    // Find the closing brace position in the first import (relative to first_span.start).
    let Some(close_brace_offset) = first_text.find('}') else {
        return Vec::new();
    };

    #[allow(clippy::as_conversions)]
    let insert_pos = first_span
        .start
        .saturating_add(u32::try_from(close_brace_offset).unwrap_or(0));

    let merged_specifiers = format!(", {}", second_specs.trim());

    vec![
        // Insert second import's specifiers before the closing brace of the first import.
        Edit {
            span: Span::new(insert_pos, insert_pos),
            replacement: merged_specifiers,
        },
        // Delete the second import statement entirely.
        delete_statement(source, second_span),
    ]
}

/// Extract the content between `{` and `}` from an import statement string.
fn extract_brace_specifiers(import_text: &str) -> Option<&str> {
    let open = import_text.find('{')?;
    let close = import_text.find('}')?;
    if open >= close {
        return None;
    }
    import_text.get(open.saturating_add(1)..close)
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    // ── source_text_for_span ──

    #[test]
    fn test_source_text_valid_span() {
        let source = "hello world";
        let text = source_text_for_span(source, Span::new(0, 5));
        assert_eq!(text, Some("hello"), "should extract 'hello'");
    }

    #[test]
    fn test_source_text_full_span() {
        let source = "hello";
        let text = source_text_for_span(source, Span::new(0, 5));
        assert_eq!(text, Some("hello"), "should extract full string");
    }

    #[test]
    fn test_source_text_out_of_bounds() {
        let source = "hi";
        let text = source_text_for_span(source, Span::new(0, 10));
        assert!(text.is_none(), "out of bounds should return None");
    }

    #[test]
    fn test_source_text_inverted() {
        let source = "hello";
        let text = source_text_for_span(source, Span::new(3, 1));
        assert!(text.is_none(), "inverted span should return None");
    }

    #[test]
    fn test_source_text_mid_utf8() {
        let source = "ä"; // 2 bytes
        let text = source_text_for_span(source, Span::new(1, 2));
        assert!(text.is_none(), "mid-UTF-8 should return None");
    }

    // ── replace / delete / insert_before ──

    #[test]
    fn test_replace_creates_edit() {
        let edit = replace(Span::new(0, 3), "const");
        assert_eq!(edit.replacement, "const", "replacement should match");
        assert_eq!(edit.span.start, 0, "start should be 0");
        assert_eq!(edit.span.end, 3, "end should be 3");
    }

    #[test]
    fn test_delete_creates_empty() {
        let edit = delete(Span::new(5, 10));
        assert!(edit.replacement.is_empty(), "delete should be empty");
    }

    #[test]
    fn test_insert_before_creates_zero_width() {
        let edit = insert_before(10, "text");
        assert_eq!(edit.span.start, 10, "start should be offset");
        assert_eq!(edit.span.end, 10, "end should equal start");
        assert_eq!(edit.replacement, "text", "replacement should match");
    }

    // ── delete_statement ──

    #[test]
    fn test_delete_statement_single_line() {
        let source = "  debugger;\n  const x = 1;";
        // "debugger;" is bytes 2..11 (exclusive end)
        let edit = delete_statement(source, Span::new(2, 11));
        assert!(
            edit.replacement.is_empty(),
            "should produce empty replacement"
        );
        // Should expand to cover "  debugger;\n" (bytes 0..12)
        assert_eq!(edit.span.start, 0, "should expand to line start");
        assert_eq!(edit.span.end, 12, "should include trailing newline");
    }

    #[test]
    fn test_delete_statement_last_line_no_newline() {
        let source = "const x = 1;\ndebugger;";
        let edit = delete_statement(source, Span::new(13, 22));
        // Should expand backward to include the newline before "debugger;"
        assert_eq!(edit.span.start, 12, "should include preceding newline");
        assert_eq!(edit.span.end, 22, "should stop at end of file");
    }

    #[test]
    fn test_delete_statement_shared_line() {
        // Statement shares a line with other code — don't expand backward
        let source = "const x = 1; debugger;";
        let edit = delete_statement(source, Span::new(13, 22));
        // Should NOT expand backward past the space since there's code before
        assert_eq!(edit.span.start, 13, "should not expand past other code");
    }

    #[test]
    fn test_delete_statement_at_file_start() {
        let source = "debugger;\nconst x = 1;";
        let edit = delete_statement(source, Span::new(0, 9));
        assert_eq!(edit.span.start, 0, "should start at 0");
        assert_eq!(edit.span.end, 10, "should include trailing newline");
    }

    // ── remove_jsx_attr ──

    #[test]
    fn test_remove_jsx_attr_with_leading_space() {
        let source = r#"<div className="x" accessKey="s">"#;
        // accessKey="s" starts at 19, ends at 32
        let edit = remove_jsx_attr(source, Span::new(19, 32));
        // Should consume the space before accessKey
        assert_eq!(edit.span.start, 18, "should include leading space");
        assert_eq!(edit.span.end, 32, "end should be same");
    }

    #[test]
    fn test_remove_jsx_attr_first_attr() {
        let source = r#"<div accessKey="s" className="x">"#;
        // accessKey="s" starts at 5, ends at 18
        let edit = remove_jsx_attr(source, Span::new(5, 18));
        // Should consume leading space
        assert_eq!(edit.span.start, 4, "should include leading space");
        assert_eq!(edit.span.end, 18, "end should be same");
    }

    // ── merge_import_edits ──

    #[test]
    fn test_merge_simple_imports() {
        let source = "import { foo } from 'mod';\nimport { bar } from 'mod';\n";
        let edits = merge_import_edits(source, Span::new(0, 25), Span::new(26, 51));
        assert_eq!(edits.len(), 2, "should produce two edits");
        // First edit: insert ", bar" before the closing brace of first import
        assert!(
            edits.first().is_some_and(|e| e.replacement.contains("bar")),
            "should insert second specifiers"
        );
        // Second edit: delete the second import statement
        assert!(
            edits.get(1).is_some_and(|e| e.replacement.is_empty()),
            "should delete second import"
        );
    }

    #[test]
    fn test_merge_no_braces_returns_empty() {
        let source = "import foo from 'mod';\nimport bar from 'mod';\n";
        let edits = merge_import_edits(source, Span::new(0, 21), Span::new(22, 43));
        assert!(
            edits.is_empty(),
            "default imports should not be merged by this utility"
        );
    }

    // ── rename_symbol_edits (basic structure test — no semantic needed) ──
    // Full integration tests require parsing + semantic builder; see rule-level tests.
}
