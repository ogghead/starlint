//! Rule: `no-restricted-imports`
//!
//! Disallow specified module imports. Useful for preventing imports from
//! deprecated packages, internal modules, or modules that should be replaced.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags import declarations from restricted modules.
#[derive(Debug)]
pub struct NoRestrictedImports {
    /// List of restricted module specifiers.
    restricted: Vec<String>,
}

impl NoRestrictedImports {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            restricted: Vec::new(),
        }
    }
}

impl Default for NoRestrictedImports {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for NoRestrictedImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-restricted-imports".to_owned(),
            description: "Disallow specified imports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(arr) = config.get("paths").and_then(serde_json::Value::as_array) {
            self.restricted = arr
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(String::from)
                .collect();
        }
        Ok(())
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if self.restricted.is_empty() {
            return;
        }

        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source = import.source.value.as_str();
        if self.restricted.iter().any(|r| r == source) {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove restricted import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "no-restricted-imports".to_owned(),
                message: format!("'{source}' import is restricted from being used"),
                span: import_span,
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedImports {
                restricted: restricted.iter().map(|s| (*s).to_owned()).collect(),
            })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_restricted_import() {
        let diags = lint_restricted("import foo from 'lodash';", &["lodash"]);
        assert_eq!(diags.len(), 1, "restricted import should be flagged");
    }

    #[test]
    fn test_allows_non_restricted() {
        let diags = lint_restricted("import foo from 'react';", &["lodash"]);
        assert!(
            diags.is_empty(),
            "non-restricted import should not be flagged"
        );
    }

    #[test]
    fn test_empty_restricted_list() {
        let diags = lint_restricted("import foo from 'lodash';", &[]);
        assert!(
            diags.is_empty(),
            "empty restricted list should flag nothing"
        );
    }

    #[test]
    fn test_flags_named_import() {
        let diags = lint_restricted("import { map } from 'lodash';", &["lodash"]);
        assert_eq!(
            diags.len(),
            1,
            "named import from restricted should be flagged"
        );
    }

    #[test]
    fn test_flags_side_effect_import() {
        let diags = lint_restricted("import 'lodash';", &["lodash"]);
        assert_eq!(
            diags.len(),
            1,
            "side-effect import from restricted should be flagged"
        );
    }
}
