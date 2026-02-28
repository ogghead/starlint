//! Rule: `no-restricted-globals`
//!
//! Disallow specified global variable names. This is commonly used to prevent
//! accidental use of browser globals like `event` or `name` that may shadow
//! local variables.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of restricted global variables.
#[derive(Debug)]
pub struct NoRestrictedGlobals {
    /// List of restricted global variable names.
    restricted: Vec<String>,
}

impl NoRestrictedGlobals {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            restricted: Vec::new(),
        }
    }
}

impl Default for NoRestrictedGlobals {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for NoRestrictedGlobals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-restricted-globals".to_owned(),
            description: "Disallow specified global variables".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(arr) = config
            .get("restricted")
            .and_then(serde_json::Value::as_array)
        {
            self.restricted = arr
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(String::from)
                .collect();
        }
        Ok(())
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if self.restricted.is_empty() {
            return;
        }

        let AstKind::IdentifierReference(ident) = kind else {
            return;
        };

        let name = ident.name.as_str();
        if self.restricted.iter().any(|r| r == name) {
            ctx.report_warning(
                "no-restricted-globals",
                &format!("Unexpected use of '{name}'"),
                Span::new(ident.span.start, ident.span.end),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint_restricted(
        source: &str,
        restricted: &[&str],
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedGlobals {
                restricted: restricted.iter().map(|s| (*s).to_owned()).collect(),
            })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_restricted_global() {
        let diags = lint_restricted("event.preventDefault();", &["event"]);
        assert_eq!(diags.len(), 1, "restricted global should be flagged");
    }

    #[test]
    fn test_allows_non_restricted() {
        let diags = lint_restricted("console.log('hello');", &["event"]);
        assert!(
            diags.is_empty(),
            "non-restricted global should not be flagged"
        );
    }

    #[test]
    fn test_empty_restricted_list() {
        let diags = lint_restricted("event.preventDefault();", &[]);
        assert!(
            diags.is_empty(),
            "empty restricted list should flag nothing"
        );
    }

    #[test]
    fn test_multiple_restricted() {
        let diags = lint_restricted("var x = name; var y = event;", &["name", "event"]);
        assert_eq!(diags.len(), 2, "both restricted globals should be flagged");
    }
}
