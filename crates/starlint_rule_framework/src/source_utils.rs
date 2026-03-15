//! Source-text scanning utilities for lint rules.
//!
//! Provides helpers for matching delimiters (braces, parentheses) and
//! extracting text spans. These are used by rules that perform
//! text-based analysis via `run_once()`.

/// Find the position of the matching closing delimiter for an opening delimiter.
///
/// Given the position of an opening delimiter (e.g., `{` or `(`), scans
/// forward through `source` counting nesting depth and returns the position
/// of the matching closing delimiter.
///
/// # Examples
///
/// ```ignore
/// // Find matching `}` for `{` at position 10
/// let close = find_matching_delimiter(source, 10, '{', '}');
///
/// // Find matching `)` for `(` at position 5
/// let close = find_matching_delimiter(source, 5, '(', ')');
/// ```
#[must_use]
pub fn find_matching_delimiter(
    source: &str,
    open_pos: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, ch) in source.get(open_pos..)?.char_indices() {
        if ch == open {
            depth = depth.saturating_add(1);
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(open_pos.saturating_add(i));
            }
        }
    }
    None
}

/// Find the position of the matching closing brace for an opening `{`.
///
/// Convenience wrapper around [`find_matching_delimiter`] for braces.
#[must_use]
pub fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    find_matching_delimiter(source, open_pos, '{', '}')
}

/// Find the position of the matching closing parenthesis for an opening `(`.
///
/// Convenience wrapper around [`find_matching_delimiter`] for parentheses.
#[must_use]
pub fn find_matching_paren(source: &str, open_pos: usize) -> Option<usize> {
    find_matching_delimiter(source, open_pos, '(', ')')
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_find_matching_brace_simple() {
        let source = "fn foo() { bar(); }";
        let result = find_matching_brace(source, 9);
        assert_eq!(result, Some(18), "should find closing brace");
    }

    #[test]
    fn test_find_matching_brace_nested() {
        let source = "{ { inner } outer }";
        let result = find_matching_brace(source, 0);
        assert_eq!(result, Some(18), "should find outermost closing brace");
    }

    #[test]
    fn test_find_matching_brace_no_match() {
        let source = "{ unclosed";
        let result = find_matching_brace(source, 0);
        assert!(result.is_none(), "should return None for unclosed brace");
    }

    #[test]
    fn test_find_matching_paren_simple() {
        let source = "foo(bar, baz)";
        let result = find_matching_paren(source, 3);
        assert_eq!(result, Some(12), "should find closing paren");
    }

    #[test]
    fn test_find_matching_delimiter_out_of_bounds() {
        let source = "short";
        let result = find_matching_delimiter(source, 100, '{', '}');
        assert!(result.is_none(), "should handle out-of-bounds start");
    }
}
