//! Rule: `max-lines-per-function`
//!
//! Enforce a maximum number of lines per function. Functions that are too
//! long are harder to understand and maintain.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Default maximum lines per function.
const DEFAULT_MAX: u32 = 50;

/// Flags functions exceeding a maximum number of lines.
#[derive(Debug)]
pub struct MaxLinesPerFunction {
    /// Maximum number of lines allowed per function.
    max: u32,
}

impl MaxLinesPerFunction {
    #[must_use]
    pub const fn new() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl Default for MaxLinesPerFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRule for MaxLinesPerFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-lines-per-function".to_owned(),
            description: "Enforce a maximum number of lines per function".to_owned(),
            category: Category::Style,
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
        let (span, name) = match node {
            AstNode::Function(f) => {
                let Some(body_id) = f.body else { return };
                let body_span = ctx.node(body_id).map_or(
                    starlint_ast::types::Span::new(0, 0),
                    starlint_ast::AstNode::span,
                );
                let fn_name =
                    f.id.and_then(|id| {
                        if let Some(AstNode::BindingIdentifier(ident)) = ctx.node(id) {
                            Some(ident.name.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "(anonymous)".to_owned());
                (body_span, fn_name)
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                let body_span = ctx.node(arrow.body).map_or(
                    starlint_ast::types::Span::new(0, 0),
                    starlint_ast::AstNode::span,
                );
                (body_span, "(arrow function)".to_owned())
            }
            _ => return,
        };

        let source = ctx.source_text();
        let start = usize::try_from(span.start).unwrap_or(0);
        let end = usize::try_from(span.end).unwrap_or(0).min(source.len());

        if let Some(body_text) = source.get(start..end) {
            let line_count = u32::try_from(body_text.lines().count()).unwrap_or(0);
            if line_count > self.max {
                ctx.report(Diagnostic {
                    rule_name: "max-lines-per-function".to_owned(),
                    message: format!(
                        "Function '{name}' has too many lines ({line_count}). Maximum allowed is {}",
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxLinesPerFunction { max })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_short_function() {
        let diags = lint_with_max("function foo() {\n  return 1;\n}", 5);
        assert!(diags.is_empty(), "short function should not be flagged");
    }

    #[test]
    fn test_flags_long_function() {
        let source = "function foo() {\n  var a = 1;\n  var b = 2;\n  var c = 3;\n  return a;\n}";
        let diags = lint_with_max(source, 3);
        assert_eq!(diags.len(), 1, "long function should be flagged");
    }

    #[test]
    fn test_allows_within_limit() {
        let source = "function foo() {\n  return 1;\n}";
        let diags = lint_with_max(source, 3);
        assert!(diags.is_empty(), "function at limit should not be flagged");
    }

    #[test]
    fn test_arrow_function() {
        let source =
            "const foo = () => {\n  var a = 1;\n  var b = 2;\n  var c = 3;\n  return a;\n};";
        let diags = lint_with_max(source, 3);
        assert_eq!(diags.len(), 1, "long arrow function should be flagged");
    }
}
