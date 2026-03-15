//! File-pattern override matching.
//!
//! Compiles glob patterns from config override blocks into a matcher
//! that can efficiently determine per-file severity overrides.

use std::collections::HashMap;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, parse_severity};

/// A single compiled override block.
struct CompiledOverride {
    /// Compiled glob matcher for the `files` patterns.
    matcher: GlobSet,
    /// Severity overrides for matching files.
    /// `None` means "off" (suppress diagnostics from that rule).
    severity_map: HashMap<String, Option<Severity>>,
}

/// Pre-compiled set of all override blocks from config.
///
/// Thread-safe (`GlobSet` is `Send + Sync`). Applied per-file after
/// diagnostics are collected.
pub struct OverrideSet {
    /// Compiled override blocks, in config order (later wins on conflict).
    overrides: Vec<CompiledOverride>,
}

impl OverrideSet {
    /// Create an empty override set (no-op).
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            overrides: Vec::new(),
        }
    }

    /// Compile override blocks from config.
    ///
    /// Invalid glob patterns are logged as warnings and skipped.
    /// Invalid severity strings are logged as warnings and skipped.
    #[must_use]
    pub fn compile(overrides: &[starlint_config::Override]) -> Self {
        let mut compiled = Vec::with_capacity(overrides.len());

        for (idx, ov) in overrides.iter().enumerate() {
            let mut builder = GlobSetBuilder::new();
            let mut valid_patterns = 0usize;

            for pattern in &ov.files {
                match Glob::new(pattern) {
                    Ok(glob) => {
                        builder.add(glob);
                        valid_patterns = valid_patterns.saturating_add(1);
                    }
                    Err(err) => {
                        tracing::warn!(
                            "override block {idx}: invalid glob pattern `{pattern}`: {err}"
                        );
                    }
                }
            }

            if valid_patterns == 0 {
                tracing::warn!("override block {idx}: no valid file patterns, skipping");
                continue;
            }

            let matcher = match builder.build() {
                Ok(set) => set,
                Err(err) => {
                    tracing::warn!("override block {idx}: failed to compile globs: {err}");
                    continue;
                }
            };

            let mut severity_map = HashMap::with_capacity(ov.rules.len());
            for (rule_name, rule_config) in &ov.rules {
                let sev_str = match rule_config {
                    starlint_config::RuleConfig::Severity(s) => s.as_str(),
                    starlint_config::RuleConfig::Detailed(d) => d.severity.as_str(),
                };
                match parse_severity(sev_str) {
                    Ok(severity) => {
                        severity_map.insert(rule_name.clone(), severity);
                    }
                    Err(err) => {
                        tracing::warn!("override block {idx}, rule `{rule_name}`: {err}");
                    }
                }
            }

            compiled.push(CompiledOverride {
                matcher,
                severity_map,
            });
        }

        Self {
            overrides: compiled,
        }
    }

    /// Returns true if there are no compiled overrides.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    /// Compute the effective severity for a rule on a given file path.
    ///
    /// Returns `Some(Some(severity))` if an override sets a severity,
    /// `Some(None)` if an override turns the rule off,
    /// `None` if no override applies for this rule and file.
    ///
    /// All matching blocks are evaluated in order; last match wins.
    #[must_use]
    pub fn effective_severity(
        &self,
        file_path: &Path,
        rule_name: &str,
    ) -> Option<Option<Severity>> {
        let mut result: Option<Option<Severity>> = None;

        for compiled_ov in &self.overrides {
            if compiled_ov.matcher.is_match(file_path) {
                if let Some(severity) = compiled_ov.severity_map.get(rule_name) {
                    result = Some(*severity);
                }
            }
        }

        result
    }

    /// Apply overrides to diagnostics for a given file path.
    ///
    /// For each matching override block (in order), adjusts severity or
    /// removes diagnostics for rules set to "off". Also removes diagnostics
    /// from `disabled_rules` unless an override explicitly activates them.
    pub fn apply(
        &self,
        file_path: &Path,
        disabled_rules: &std::collections::HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.overrides.is_empty() && disabled_rules.is_empty() {
            return;
        }

        diagnostics.retain_mut(
            |diag| match self.effective_severity(file_path, &diag.rule_name) {
                Some(Some(severity)) => {
                    diag.severity = severity;
                    true
                }
                Some(None) => false,
                None => !disabled_rules.contains(&diag.rule_name),
            },
        );
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // Test assertions on known-length vectors
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use std::collections::HashSet;

    /// Helper to build a simple override config block.
    fn make_override(files: &[&str], rules: &[(&str, &str)]) -> starlint_config::Override {
        let rule_map = rules
            .iter()
            .map(|(name, sev)| {
                (
                    (*name).to_owned(),
                    starlint_config::RuleConfig::Severity((*sev).to_owned()),
                )
            })
            .collect();
        starlint_config::Override {
            files: files.iter().map(|s| (*s).to_owned()).collect(),
            rules: rule_map,
        }
    }

    /// Helper to build a test diagnostic.
    fn make_diag(rule: &str, severity: Severity) -> Diagnostic {
        Diagnostic {
            rule_name: rule.to_owned(),
            message: format!("test {rule}"),
            span: starlint_plugin_sdk::diagnostic::Span::new(0, 0),
            severity,
            help: None,
            fix: None,
            labels: vec![],
        }
    }

    #[test]
    fn test_empty_override_set() {
        let set = OverrideSet::empty();
        assert!(set.is_empty(), "empty set should be empty");
        assert_eq!(
            set.effective_severity(Path::new("test.ts"), "no-debugger"),
            None,
            "empty set should return None"
        );
    }

    #[test]
    fn test_compile_single_block() {
        let overrides = vec![make_override(
            &["**/*.test.ts"],
            &[("no-debugger", "off"), ("no-console", "warn")],
        )];
        let set = OverrideSet::compile(&overrides);
        assert!(!set.is_empty(), "should have one compiled block");
    }

    #[test]
    fn test_effective_severity_matching_file() {
        let overrides = vec![make_override(
            &["**/*.test.ts"],
            &[("no-debugger", "off"), ("no-console", "warn")],
        )];
        let set = OverrideSet::compile(&overrides);

        assert_eq!(
            set.effective_severity(Path::new("src/foo.test.ts"), "no-debugger"),
            Some(None),
            "no-debugger should be off for test files"
        );
        assert_eq!(
            set.effective_severity(Path::new("src/foo.test.ts"), "no-console"),
            Some(Some(Severity::Warning)),
            "no-console should be warn for test files"
        );
    }

    #[test]
    fn test_effective_severity_non_matching_file() {
        let overrides = vec![make_override(&["**/*.test.ts"], &[("no-debugger", "off")])];
        let set = OverrideSet::compile(&overrides);

        assert_eq!(
            set.effective_severity(Path::new("src/foo.ts"), "no-debugger"),
            None,
            "non-matching file should return None"
        );
    }

    #[test]
    fn test_effective_severity_unmentioned_rule() {
        let overrides = vec![make_override(&["**/*.test.ts"], &[("no-debugger", "off")])];
        let set = OverrideSet::compile(&overrides);

        assert_eq!(
            set.effective_severity(Path::new("src/foo.test.ts"), "no-console"),
            None,
            "rule not in override should return None"
        );
    }

    #[test]
    fn test_last_matching_override_wins() {
        let overrides = vec![
            make_override(&["**/*.ts"], &[("no-debugger", "warn")]),
            make_override(&["**/*.test.ts"], &[("no-debugger", "off")]),
        ];
        let set = OverrideSet::compile(&overrides);

        // test.ts matches both blocks — last wins
        assert_eq!(
            set.effective_severity(Path::new("src/foo.test.ts"), "no-debugger"),
            Some(None),
            "last matching block should win (off)"
        );

        // plain .ts matches only first block
        assert_eq!(
            set.effective_severity(Path::new("src/foo.ts"), "no-debugger"),
            Some(Some(Severity::Warning)),
            "first block matches plain .ts (warn)"
        );
    }

    #[test]
    fn test_apply_changes_severity() {
        let overrides = vec![make_override(&["**/*.test.ts"], &[("no-console", "warn")])];
        let set = OverrideSet::compile(&overrides);

        let mut diags = vec![make_diag("no-console", Severity::Error)];
        set.apply(Path::new("src/foo.test.ts"), &HashSet::new(), &mut diags);

        assert_eq!(diags.len(), 1, "diagnostic should be retained");
        assert_eq!(
            diags[0].severity,
            Severity::Warning,
            "severity should be changed to warn"
        );
    }

    #[test]
    fn test_apply_removes_off_diagnostics() {
        let overrides = vec![make_override(&["**/*.test.ts"], &[("no-debugger", "off")])];
        let set = OverrideSet::compile(&overrides);

        let mut diags = vec![
            make_diag("no-debugger", Severity::Error),
            make_diag("no-console", Severity::Error),
        ];
        set.apply(Path::new("src/foo.test.ts"), &HashSet::new(), &mut diags);

        assert_eq!(diags.len(), 1, "off rule should be removed");
        assert_eq!(diags[0].rule_name, "no-console", "no-console should remain");
    }

    #[test]
    fn test_apply_non_matching_file_unchanged() {
        let overrides = vec![make_override(&["**/*.test.ts"], &[("no-debugger", "off")])];
        let set = OverrideSet::compile(&overrides);

        let mut diags = vec![make_diag("no-debugger", Severity::Error)];
        set.apply(Path::new("src/foo.ts"), &HashSet::new(), &mut diags);

        assert_eq!(diags.len(), 1, "non-matching file should be unchanged");
        assert_eq!(
            diags[0].severity,
            Severity::Error,
            "severity should remain Error"
        );
    }

    #[test]
    fn test_apply_suppresses_disabled_rules() {
        let set = OverrideSet::empty();

        let mut disabled = HashSet::new();
        disabled.insert("no-console".to_owned());

        let mut diags = vec![
            make_diag("no-debugger", Severity::Error),
            make_diag("no-console", Severity::Error),
        ];
        set.apply(Path::new("src/foo.ts"), &disabled, &mut diags);

        assert_eq!(diags.len(), 1, "disabled rule should be suppressed");
        assert_eq!(
            diags[0].rule_name, "no-debugger",
            "non-disabled rule should remain"
        );
    }

    #[test]
    fn test_apply_override_activates_disabled_rule() {
        let overrides = vec![make_override(&["**/*.test.ts"], &[("no-console", "warn")])];
        let set = OverrideSet::compile(&overrides);

        let mut disabled = HashSet::new();
        disabled.insert("no-console".to_owned());

        let mut diags = vec![make_diag("no-console", Severity::Error)];
        set.apply(Path::new("src/foo.test.ts"), &disabled, &mut diags);

        assert_eq!(diags.len(), 1, "override should activate disabled rule");
        assert_eq!(
            diags[0].severity,
            Severity::Warning,
            "severity should be changed to warn"
        );
    }

    #[test]
    fn test_compile_invalid_glob_skipped() {
        // A pattern with unmatched bracket is invalid
        let overrides = vec![make_override(&["[invalid"], &[("no-debugger", "off")])];
        let set = OverrideSet::compile(&overrides);
        assert!(
            set.is_empty(),
            "invalid glob should result in skipped block"
        );
    }

    #[test]
    fn test_compile_invalid_severity_skipped() {
        let overrides = vec![make_override(&["**/*.ts"], &[("no-debugger", "badvalue")])];
        let set = OverrideSet::compile(&overrides);
        assert!(!set.is_empty(), "block should still compile");
        assert_eq!(
            set.effective_severity(Path::new("src/foo.ts"), "no-debugger"),
            None,
            "invalid severity should be skipped"
        );
    }
}
