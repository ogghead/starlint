//! Rule: `max-statements` (eslint)
//!
//! Flag functions with too many statements. Functions with many statements
//! are harder to understand and should be broken into smaller pieces.

use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Default maximum number of statements per function.
const DEFAULT_MAX: u32 = 10;

/// Flags functions with too many statements.
#[derive(Debug)]
pub struct MaxStatements {
    /// Maximum number of statements allowed per function.
    max: RwLock<u32>,
}

impl MaxStatements {
    /// Create a new `MaxStatements` rule with the default threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max: RwLock::new(DEFAULT_MAX),
        }
    }
}

impl Default for MaxStatements {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRule for MaxStatements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-statements".to_owned(),
            description: "Enforce a maximum number of statements per function".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            let val = u32::try_from(n).unwrap_or(DEFAULT_MAX);
            if let Ok(mut guard) = self.max.write() {
                *guard = val;
            }
        }
        Ok(())
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let threshold = self.max.read().map_or(DEFAULT_MAX, |g| *g);

        let (stmt_count, span, name) = match node {
            AstNode::Function(f) => {
                let Some(body_id) = f.body else { return };
                let count = match ctx.node(body_id) {
                    Some(AstNode::FunctionBody(body)) => {
                        u32::try_from(body.statements.len()).unwrap_or(0)
                    }
                    _ => return,
                };
                let fn_name =
                    f.id.and_then(|id| {
                        if let Some(AstNode::BindingIdentifier(ident)) = ctx.node(id) {
                            Some(ident.name.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "(anonymous)".to_owned());
                (count, f.span, fn_name)
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                let count = match ctx.node(arrow.body) {
                    Some(AstNode::FunctionBody(body)) => {
                        u32::try_from(body.statements.len()).unwrap_or(0)
                    }
                    _ => 0,
                };
                (count, arrow.span, "(arrow function)".to_owned())
            }
            _ => return,
        };

        if stmt_count > threshold {
            ctx.report(Diagnostic {
                rule_name: "max-statements".to_owned(),
                message: format!(
                    "Function '{name}' has too many statements ({stmt_count}). Maximum allowed is {threshold}"
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
    use starlint_rule_framework::lint_source;

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxStatements {
            max: RwLock::new(max),
        })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_few_statements() {
        let source = "function foo() { var a = 1; var b = 2; var c = 3; }";
        let diags = lint_with_max(source, 10);
        assert!(
            diags.is_empty(),
            "function with few statements should not be flagged"
        );
    }

    #[test]
    fn test_flags_many_statements() {
        let source = r"function foo() {
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
            var e = 5;
            var f = 6;
            var g = 7;
            var h = 8;
            var i = 9;
            var j = 10;
            var k = 11;
        }";
        let diags = lint_with_max(source, 10);
        assert_eq!(
            diags.len(),
            1,
            "function with many statements should be flagged"
        );
    }

    #[test]
    fn test_allows_at_limit() {
        let source = r"function foo() {
            var a = 1;
            var b = 2;
            var c = 3;
        }";
        let diags = lint_with_max(source, 3);
        assert!(
            diags.is_empty(),
            "function at the limit should not be flagged"
        );
    }

    #[test]
    fn test_arrow_function_flagged() {
        let source = r"const foo = () => {
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
        };";
        let diags = lint_with_max(source, 3);
        assert_eq!(
            diags.len(),
            1,
            "arrow function with too many statements should be flagged"
        );
    }
}
