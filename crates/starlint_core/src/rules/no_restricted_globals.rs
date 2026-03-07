//! Rule: `no-restricted-globals`
//!
//! Disallow specified global variable names. This is commonly used to prevent
//! accidental use of browser globals like `event` or `name` that may shadow
//! local variables.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

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

impl LintRule for NoRestrictedGlobals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-restricted-globals".to_owned(),
            description: "Disallow specified global variables".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IdentifierReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if self.restricted.is_empty() {
            return;
        }

        let AstNode::IdentifierReference(ident) = node else {
            return;
        };

        let name = ident.name.as_str();
        if self.restricted.iter().any(|r| r == name) {
            ctx.report(Diagnostic {
                rule_name: "no-restricted-globals".to_owned(),
                message: format!("Unexpected use of '{name}'"),
                span: Span::new(ident.span.start, ident.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint_restricted(source: &str, restricted: &[&str]) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRestrictedGlobals {
            restricted: restricted.iter().map(|s| (*s).to_owned()).collect(),
        })];
        lint_source(source, "test.js", &rules)
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
