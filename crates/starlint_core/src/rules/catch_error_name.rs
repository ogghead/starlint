//! Rule: `catch-error-name`
//!
//! Enforce a consistent parameter name in catch clauses. By default, the
//! expected name is `error`. This improves grep-ability and consistency
//! across a codebase.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Default expected catch parameter name.
const DEFAULT_NAME: &str = "error";

/// Flags catch clauses whose parameter name doesn't match the expected name.
#[derive(Debug)]
pub struct CatchErrorName {
    /// The expected catch parameter name.
    expected_name: String,
}

impl Default for CatchErrorName {
    fn default() -> Self {
        Self::new()
    }
}

impl CatchErrorName {
    /// Create a new rule with the default expected name (`error`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            expected_name: DEFAULT_NAME.to_owned(),
        }
    }
}

impl LintRule for CatchErrorName {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "catch-error-name".to_owned(),
            description: "Enforce a consistent parameter name in catch clauses".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(name) = config.get("name").and_then(serde_json::Value::as_str) {
            name.clone_into(&mut self.expected_name);
        }
        Ok(())
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CatchClause])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CatchClause(clause) = node else {
            return;
        };

        // No param → nothing to check (`catch {}`)
        let Some(param_id) = clause.param else {
            return;
        };

        // Only check simple identifier params, skip destructured patterns
        let Some(AstNode::BindingIdentifier(id)) = ctx.node(param_id) else {
            return;
        };

        let name = id.name.as_str();

        // Allow `_` (intentionally unused convention)
        if name == "_" {
            return;
        }

        if name == self.expected_name {
            return;
        }

        // Extract data before mutable borrow of ctx
        let id_span = Span::new(id.span.start, id.span.end);
        let name_owned = name.to_owned();
        let expected = self.expected_name.clone();
        ctx.report(Diagnostic {
            rule_name: "catch-error-name".to_owned(),
            message: format!("Catch parameter should be named `{expected}`"),
            span: id_span,
            severity: Severity::Warning,
            help: Some(format!("Rename `{name_owned}` to `{expected}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Rename to `{expected}`"),
                edits: vec![Edit {
                    span: id_span,
                    replacement: expected,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CatchErrorName::new())];
        lint_source(source, "test.js", &rules)
    }

    fn lint_with_name(source: &str, expected: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CatchErrorName {
            expected_name: expected.to_owned(),
        })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_short_name() {
        let diags = lint("try {} catch (e) {}");
        assert_eq!(diags.len(), 1, "should flag 'e'");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("`error`")),
            "message should suggest 'error'"
        );
    }

    #[test]
    fn test_flags_err() {
        let diags = lint("try {} catch (err) {}");
        assert_eq!(diags.len(), 1, "should flag 'err'");
    }

    #[test]
    fn test_flags_ex() {
        let diags = lint("try {} catch (ex) {}");
        assert_eq!(diags.len(), 1, "should flag 'ex'");
    }

    #[test]
    fn test_allows_error() {
        let diags = lint("try {} catch (error) {}");
        assert!(diags.is_empty(), "'error' should not be flagged");
    }

    #[test]
    fn test_allows_underscore() {
        let diags = lint("try {} catch (_) {}");
        assert!(diags.is_empty(), "'_' should not be flagged");
    }

    #[test]
    fn test_allows_no_param() {
        let diags = lint("try {} catch {}");
        assert!(diags.is_empty(), "no param should not be flagged");
    }

    #[test]
    fn test_allows_destructured() {
        let diags = lint("try {} catch ({ message }) {}");
        assert!(diags.is_empty(), "destructured should not be flagged");
    }

    #[test]
    fn test_configure_custom_name_allows() {
        let diags = lint_with_name("try {} catch (err) {}", "err");
        assert!(diags.is_empty(), "'err' should pass when configured");
    }

    #[test]
    fn test_configure_custom_name_flags() {
        let diags = lint_with_name("try {} catch (error) {}", "err");
        assert_eq!(
            diags.len(),
            1,
            "'error' should fail when 'err' is configured"
        );
    }

    #[test]
    fn test_configure_via_method() {
        let mut rule = CatchErrorName::new();
        let config = serde_json::json!({ "name": "ex" });
        assert!(rule.configure(&config).is_ok());
        assert_eq!(rule.expected_name, "ex", "name should be updated");
    }
}
