//! Rule: `max-params` (eslint)
//!
//! Flag functions with too many parameters. Functions with many parameters
//! are harder to call correctly — prefer using an options object instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Default maximum number of parameters.
const DEFAULT_MAX: u32 = 3;

/// Flags functions with too many parameters.
#[derive(Debug)]
pub struct MaxParams {
    /// Maximum number of parameters allowed per function.
    max: u32,
}

impl MaxParams {
    /// Create a new `MaxParams` rule with the default threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl Default for MaxParams {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRule for MaxParams {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-params".to_owned(),
            description: "Enforce a maximum number of parameters per function".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = u32::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let (param_count, span, name) = match node {
            AstNode::Function(f) => {
                let count = u32::try_from(f.params.len()).unwrap_or(0);
                let fn_name = f
                    .id
                    .and_then(|id| ctx.node(id))
                    .and_then(|n| n.as_binding_identifier())
                    .map_or_else(|| "(anonymous)".to_owned(), |bi| bi.name.clone());
                (count, f.span, fn_name)
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                let count = u32::try_from(arrow.params.len()).unwrap_or(0);
                (count, arrow.span, "(arrow function)".to_owned())
            }
            _ => return,
        };

        if param_count > self.max {
            ctx.report(Diagnostic {
                rule_name: "max-params".to_owned(),
                message: format!(
                    "Function '{name}' has too many parameters ({param_count}). Maximum allowed is {}",
                    self.max
                ),
                span: Span::new(span.start, span.end),
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

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxParams { max })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_few_params() {
        let diags = lint_with_max("function foo(a, b) {}", 3);
        assert!(
            diags.is_empty(),
            "function with few params should not be flagged"
        );
    }

    #[test]
    fn test_flags_too_many_params() {
        let diags = lint_with_max("function foo(a, b, c, d) {}", 3);
        assert_eq!(
            diags.len(),
            1,
            "function with too many params should be flagged"
        );
    }

    #[test]
    fn test_allows_at_limit() {
        let diags = lint_with_max("function foo(a, b, c) {}", 3);
        assert!(
            diags.is_empty(),
            "function at param limit should not be flagged"
        );
    }

    #[test]
    fn test_arrow_function() {
        let diags = lint_with_max("const foo = (a, b, c, d) => {};", 3);
        assert_eq!(
            diags.len(),
            1,
            "arrow function with too many params should be flagged"
        );
    }

    #[test]
    fn test_allows_no_params() {
        let diags = lint_with_max("function foo() {}", 3);
        assert!(
            diags.is_empty(),
            "function with no params should not be flagged"
        );
    }
}
