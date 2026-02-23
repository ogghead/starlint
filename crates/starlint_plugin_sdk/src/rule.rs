//! Rule metadata types.
//!
//! Describes a lint rule's identity, category, and fix capabilities.

use serde::{Deserialize, Serialize};

use crate::diagnostic::Severity;

/// Category of a lint rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Rules that catch likely bugs or incorrect behavior.
    Correctness,
    /// Rules that enforce code style conventions.
    Style,
    /// Rules that suggest performance improvements.
    Performance,
    /// Rules that provide improvement suggestions.
    Suggestion,
    /// A plugin-defined custom category.
    Custom(String),
}

/// Whether and how a rule can auto-fix issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixKind {
    /// Rule does not provide fixes.
    None,
    /// Fix is safe and preserves semantics.
    SafeFix,
    /// Fix is a suggestion that may change semantics.
    SuggestionFix,
    /// Fix may change semantics in a breaking way.
    DangerousFix,
}

/// Metadata describing a lint rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleMeta {
    /// Fully qualified rule name (e.g. "storybook/default-exports").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Rule category.
    pub category: Category,
    /// Default severity when not overridden by config.
    pub default_severity: Severity,
    /// Whether the rule can auto-fix.
    pub fix_kind: FixKind,
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_rule_meta_serialization() {
        let meta = RuleMeta {
            name: "test/no-debugger".to_owned(),
            description: "Disallow debugger statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        };

        let json = serde_json::to_string(&meta).ok();
        assert!(json.is_some(), "serialization should succeed");
    }

    #[test]
    fn test_custom_category() {
        let cat = Category::Custom("storybook".to_owned());
        let json = serde_json::to_string(&cat).ok();
        assert!(json.is_some(), "custom category should serialize");
    }
}
