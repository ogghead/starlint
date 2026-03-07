//! Rule: `no-restricted-imports`
//!
//! Disallow specified module imports. Useful for preventing imports from
//! deprecated packages, internal modules, or modules that should be replaced.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

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

impl LintRule for NoRestrictedImports {
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

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if self.restricted.is_empty() {
            return;
        }

        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        let source = import.source.as_str();
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
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint_restricted(
        source: &str,
        restricted: &[&str],
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRestrictedImports {
            restricted: restricted.iter().map(|s| (*s).to_owned()).collect(),
        })];
        lint_source(source, "test.js", &rules)
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
