//! Result caching for starlint.
//!
//! Caches lint results per file based on a content hash. On subsequent runs,
//! files whose content hash matches the cache entry are skipped, saving parsing
//! and rule execution time.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// A cache entry for a single file.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Hex-encoded content hash.
    content_hash: String,
    /// Number of errors found.
    errors: u32,
    /// Number of warnings found.
    warnings: u32,
}

/// File-level lint result cache.
///
/// Stores content hashes and diagnostic counts per file. When a file's content
/// hash matches the cached value, its diagnostics can be skipped.
#[derive(Debug)]
pub struct LintCache {
    /// Cache entries keyed by file path (as string for portability).
    entries: HashMap<String, CacheEntry>,
    /// Whether the cache was modified since loading.
    dirty: bool,
}

impl LintCache {
    /// Create an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            dirty: false,
        }
    }

    /// Load a cache from disk. Returns an empty cache if the file doesn't exist
    /// or is malformed.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        let Ok(content) = std::fs::read_to_string(path) else {
            return Self::new();
        };

        let mut entries = HashMap::new();
        for raw_line in content.as_bytes().lines() {
            let Ok(line) = raw_line else {
                continue;
            };
            // Format: hash\terrors\twarnings\tpath
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() < 4 {
                continue;
            }
            let Some(hash) = parts.first() else {
                continue;
            };
            let errors = parts
                .get(1)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let warnings = parts
                .get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let Some(file_path) = parts.get(3) else {
                continue;
            };
            entries.insert(
                (*file_path).to_owned(),
                CacheEntry {
                    content_hash: (*hash).to_owned(),
                    errors,
                    warnings,
                },
            );
        }

        Self {
            entries,
            dirty: false,
        }
    }

    /// Save the cache to disk.
    ///
    /// Only writes if the cache was modified since loading.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the file cannot be written.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let mut output = String::new();
        for (file_path, entry) in &self.entries {
            output.push_str(&entry.content_hash);
            output.push('\t');
            output.push_str(&entry.errors.to_string());
            output.push('\t');
            output.push_str(&entry.warnings.to_string());
            output.push('\t');
            output.push_str(file_path);
            output.push('\n');
        }

        std::fs::write(path, output)
    }

    /// Check if a file is unchanged since the last lint run.
    ///
    /// Returns `Some((errors, warnings))` if the file is cached and unchanged,
    /// `None` if it needs to be re-linted.
    #[must_use]
    pub fn check(&self, file_path: &Path, content: &str) -> Option<(u32, u32)> {
        let key = file_path.display().to_string();
        let entry = self.entries.get(&key)?;
        let hash = content_hash(content);
        (entry.content_hash == hash).then_some((entry.errors, entry.warnings))
    }

    /// Update the cache entry for a file after linting.
    pub fn update(&mut self, file_path: &Path, content: &str, errors: u32, warnings: u32) {
        let key = file_path.display().to_string();
        let hash = content_hash(content);
        self.entries.insert(
            key,
            CacheEntry {
                content_hash: hash,
                errors,
                warnings,
            },
        );
        self.dirty = true;
    }

    /// Filter a list of files to only those that need re-linting.
    ///
    /// Reads each file and checks the cache. Files that are cached and unchanged
    /// are excluded. Returns the filtered list and the cached counts.
    #[must_use]
    pub fn filter_unchanged(&self, files: &[PathBuf]) -> (Vec<PathBuf>, CachedCounts) {
        let mut needs_lint = Vec::new();
        let mut cached_errors = 0u32;
        let mut cached_warnings = 0u32;

        for path in files {
            let Ok(content) = std::fs::read_to_string(path) else {
                needs_lint.push(path.clone());
                continue;
            };
            if let Some((errors, warnings)) = self.check(path, &content) {
                cached_errors = cached_errors.saturating_add(errors);
                cached_warnings = cached_warnings.saturating_add(warnings);
            } else {
                needs_lint.push(path.clone());
            }
        }

        (
            needs_lint,
            CachedCounts {
                errors: cached_errors,
                warnings: cached_warnings,
            },
        )
    }
}

impl Default for LintCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic counts from cached (skipped) files.
#[derive(Debug, Clone, Copy, Default)]
pub struct CachedCounts {
    /// Cached error count.
    pub errors: u32,
    /// Cached warning count.
    pub warnings: u32,
}

/// Compute a simple content hash (FNV-1a 64-bit).
///
/// Not cryptographic — just for change detection. Fast and collision-resistant
/// enough for cache keys.
fn content_hash(content: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in content.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_content_hash_deterministic() {
        let h1 = content_hash("hello world");
        let h2 = content_hash("hello world");
        assert_eq!(h1, h2, "same content should produce same hash");
    }

    #[test]
    fn test_content_hash_different_content() {
        let h1 = content_hash("hello");
        let h2 = content_hash("world");
        assert_ne!(h1, h2, "different content should produce different hashes");
    }

    #[test]
    fn test_cache_new_empty() {
        let cache = LintCache::new();
        assert!(
            cache.check(Path::new("test.js"), "content").is_none(),
            "new cache should have no entries"
        );
    }

    #[test]
    fn test_cache_update_and_check() {
        let mut cache = LintCache::new();
        let path = Path::new("test.js");
        let content = "const x = 1;";

        cache.update(path, content, 2, 1);

        let result = cache.check(path, content);
        assert_eq!(
            result,
            Some((2, 1)),
            "cached file with same content should return counts"
        );
    }

    #[test]
    fn test_cache_check_changed_content() {
        let mut cache = LintCache::new();
        let path = Path::new("test.js");

        cache.update(path, "const x = 1;", 1, 0);

        let result = cache.check(path, "const x = 2;");
        assert!(result.is_none(), "changed content should not match cache");
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)] // Test cleanup is best-effort
    fn test_cache_save_and_load() {
        let dir = std::env::temp_dir().join("starlint-cache-test");
        let _ = std::fs::create_dir_all(&dir);
        let cache_path = dir.join(".starlintcache");

        let mut cache = LintCache::new();
        cache.update(Path::new("a.js"), "aaa", 1, 2);
        cache.update(Path::new("b.js"), "bbb", 0, 0);

        let save_result = cache.save(&cache_path);
        assert!(save_result.is_ok(), "save should succeed");

        let loaded = LintCache::load(&cache_path);
        assert_eq!(
            loaded.check(Path::new("a.js"), "aaa"),
            Some((1, 2)),
            "loaded cache should have entry for a.js"
        );
        assert_eq!(
            loaded.check(Path::new("b.js"), "bbb"),
            Some((0, 0)),
            "loaded cache should have entry for b.js"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cache_load_nonexistent() {
        let cache = LintCache::load(Path::new("/nonexistent/cache/file"));
        assert!(
            cache.check(Path::new("test.js"), "x").is_none(),
            "loading nonexistent file should produce empty cache"
        );
    }

    #[test]
    fn test_cache_load_malformed() {
        let dir = std::env::temp_dir().join("starlint-cache-malformed");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::create_dir_all(&dir);
        let cache_path = dir.join(".starlintcache");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::write(&cache_path, "not\tvalid\nformat");

        let cache = LintCache::load(&cache_path);
        // Should not crash, just produce empty/partial cache.
        assert!(cache.entries.is_empty() || cache.entries.len() <= 1);

        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cache_save_not_dirty() {
        let cache = LintCache::new();
        let result = cache.save(Path::new("/tmp/should-not-write"));
        assert!(result.is_ok(), "non-dirty cache should be a no-op save");
    }

    #[test]
    fn test_filter_unchanged() {
        let dir = std::env::temp_dir().join("starlint-cache-filter");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::create_dir_all(&dir);

        let file_a = dir.join("a.js");
        let file_b = dir.join("b.js");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::write(&file_a, "aaa");
        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::write(&file_b, "bbb");

        let mut cache = LintCache::new();
        cache.update(&file_a, "aaa", 1, 0);
        // b.js is not cached

        let (needs_lint, cached) = cache.filter_unchanged(&[file_a.clone(), file_b.clone()]);

        assert_eq!(needs_lint.len(), 1, "only b.js should need linting");
        assert_eq!(needs_lint.first(), Some(&file_b));
        assert_eq!(cached.errors, 1);
        assert_eq!(cached.warnings, 0);

        #[allow(clippy::let_underscore_must_use)]
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_content_hash_empty() {
        let hash = content_hash("");
        assert_eq!(hash.len(), 16, "hash should be 16 hex characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hash should contain only hex characters"
        );
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)]
    fn test_cache_load_valid_format() {
        let dir = std::env::temp_dir().join("starlint-cache-load-valid");
        let _ = std::fs::create_dir_all(&dir);
        let cache_path = dir.join(".starlintcache");

        let hash = content_hash("hello");
        let content = format!("{hash}\t3\t1\ttest.js\n");
        let _ = std::fs::write(&cache_path, content);

        let cache = LintCache::load(&cache_path);
        assert_eq!(
            cache.check(Path::new("test.js"), "hello"),
            Some((3, 1)),
            "loaded entry should match hash with correct counts"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)]
    fn test_cache_load_partial_lines() {
        let dir = std::env::temp_dir().join("starlint-cache-partial");
        let _ = std::fs::create_dir_all(&dir);
        let cache_path = dir.join(".starlintcache");

        let hash = content_hash("good");
        // First line has only 3 fields (invalid), second is valid
        let content = format!("badhash\t1\tonly_three\n{hash}\t0\t0\tgood.js\n");
        let _ = std::fs::write(&cache_path, content);

        let cache = LintCache::load(&cache_path);
        assert!(
            cache.check(Path::new("only_three"), "anything").is_none(),
            "partial line should be skipped"
        );
        assert_eq!(
            cache.check(Path::new("good.js"), "good"),
            Some((0, 0)),
            "valid line should be loaded"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[allow(clippy::let_underscore_must_use)]
    fn test_cache_load_non_numeric_counts() {
        let dir = std::env::temp_dir().join("starlint-cache-nonnumeric");
        let _ = std::fs::create_dir_all(&dir);
        let cache_path = dir.join(".starlintcache");

        let hash = content_hash("data");
        let content = format!("{hash}\tabc\txyz\tfile.js\n");
        let _ = std::fs::write(&cache_path, content);

        let cache = LintCache::load(&cache_path);
        assert_eq!(
            cache.check(Path::new("file.js"), "data"),
            Some((0, 0)),
            "non-numeric counts should default to 0"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cache_default_impl() {
        let default_cache = LintCache::default();
        let new_cache = LintCache::new();
        assert!(
            default_cache.entries.is_empty() && new_cache.entries.is_empty(),
            "default and new should both be empty"
        );
        assert!(
            !default_cache.dirty && !new_cache.dirty,
            "default and new should both be clean"
        );
    }

    #[test]
    fn test_cached_counts_default() {
        let counts = CachedCounts::default();
        assert_eq!(counts.errors, 0, "default errors should be 0");
        assert_eq!(counts.warnings, 0, "default warnings should be 0");
    }

    #[test]
    fn test_filter_unchanged_unreadable_file() {
        let cache = LintCache::new();
        let nonexistent = PathBuf::from("/nonexistent/path/to/file.js");
        let (needs_lint, cached) = cache.filter_unchanged(std::slice::from_ref(&nonexistent));
        assert_eq!(
            needs_lint.len(),
            1,
            "unreadable file should be included in needs_lint"
        );
        assert_eq!(needs_lint.first(), Some(&nonexistent));
        assert_eq!(cached.errors, 0);
        assert_eq!(cached.warnings, 0);
    }

    #[test]
    fn test_cache_update_overwrites() {
        let mut cache = LintCache::new();
        let path = Path::new("overwrite.js");

        cache.update(path, "v1", 5, 3);
        assert_eq!(cache.check(path, "v1"), Some((5, 3)));

        cache.update(path, "v2", 1, 0);
        assert!(
            cache.check(path, "v1").is_none(),
            "old content should no longer match"
        );
        assert_eq!(
            cache.check(path, "v2"),
            Some((1, 0)),
            "new content should match with updated counts"
        );
    }
}
