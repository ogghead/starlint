//! File discovery using the `ignore` crate.
//!
//! Walks directories respecting `.gitignore` and filters for JS/TS files.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

/// Default file extensions to lint.
const DEFAULT_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];

/// Discover JS/TS files in the given paths.
///
/// Respects `.gitignore` rules. Filters by file extension.
/// Returns sorted, deduplicated file paths.
pub fn discover_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_lintable_file(path) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            let walker = WalkBuilder::new(path)
                .hidden(false)
                .git_ignore(true)
                .git_global(true)
                .follow_links(false)
                .build();

            for entry in walker.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() && is_lintable_file(entry_path) {
                    files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

/// Check if a file has a lintable extension.
fn is_lintable_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| DEFAULT_EXTENSIONS.contains(&ext))
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_is_lintable_file() {
        assert!(
            is_lintable_file(Path::new("foo.ts")),
            "ts should be lintable"
        );
        assert!(
            is_lintable_file(Path::new("foo.tsx")),
            "tsx should be lintable"
        );
        assert!(
            is_lintable_file(Path::new("foo.js")),
            "js should be lintable"
        );
        assert!(
            is_lintable_file(Path::new("foo.jsx")),
            "jsx should be lintable"
        );
        assert!(
            is_lintable_file(Path::new("foo.mjs")),
            "mjs should be lintable"
        );
        assert!(
            !is_lintable_file(Path::new("foo.py")),
            "py should not be lintable"
        );
        assert!(
            !is_lintable_file(Path::new("foo.rs")),
            "rs should not be lintable"
        );
        assert!(
            !is_lintable_file(Path::new("foo")),
            "no extension should not be lintable"
        );
    }
}
