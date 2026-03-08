//! Rule: `max-nested-callbacks`
//!
//! Enforce a maximum depth of nested callbacks. Deeply nested callbacks
//! (a.k.a. "callback hell") make code hard to read and maintain.

use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Default maximum nesting depth for callbacks.
const DEFAULT_MAX: u32 = 10;

/// Flags callback functions that are nested too deeply.
#[derive(Debug)]
pub struct MaxNestedCallbacks {
    /// Maximum allowed nesting depth.
    max: u32,
    /// Current nesting depth during traversal.
    depth: RwLock<u32>,
}

impl MaxNestedCallbacks {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max: DEFAULT_MAX,
            depth: RwLock::new(0),
        }
    }
}

impl Default for MaxNestedCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRule for MaxNestedCallbacks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-nested-callbacks".to_owned(),
            description: "Enforce a maximum depth of nested callbacks".to_owned(),
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

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ArrowFunctionExpression, AstNodeType::Function])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Track callback nesting: a callback is a function/arrow that is
        // an argument to a call expression. We approximate this by tracking
        // all arrow/function expressions.
        match node {
            AstNode::ArrowFunctionExpression(arrow) => {
                let Ok(mut depth) = self.depth.write() else {
                    return;
                };
                *depth = depth.saturating_add(1);
                if *depth > self.max {
                    ctx.report(Diagnostic {
                        rule_name: "max-nested-callbacks".to_owned(),
                        message: format!(
                            "Too many nested callbacks ({depth}). Maximum allowed is {}",
                            self.max
                        ),
                        span: Span::new(arrow.span.start, arrow.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::Function(f) if f.id.is_none() => {
                // Only count anonymous functions (callbacks), not declarations
                let Ok(mut depth) = self.depth.write() else {
                    return;
                };
                *depth = depth.saturating_add(1);
                if *depth > self.max {
                    ctx.report(Diagnostic {
                        rule_name: "max-nested-callbacks".to_owned(),
                        message: format!(
                            "Too many nested callbacks ({depth}). Maximum allowed is {}",
                            self.max
                        ),
                        span: Span::new(f.span.start, f.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }

    fn leave(&self, _node_id: NodeId, node: &AstNode, _ctx: &mut LintContext<'_>) {
        match node {
            AstNode::ArrowFunctionExpression(_) => {
                if let Ok(mut depth) = self.depth.write() {
                    *depth = depth.saturating_sub(1);
                }
            }
            AstNode::Function(f) if f.id.is_none() => {
                if let Ok(mut depth) = self.depth.write() {
                    *depth = depth.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxNestedCallbacks {
            max,
            depth: RwLock::new(0),
        })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_within_limit() {
        let diags = lint_with_max("foo(() => { bar(() => {}); });", 2);
        assert!(
            diags.is_empty(),
            "2 levels with max=2 should not be flagged"
        );
    }

    #[test]
    fn test_flags_exceeding_limit() {
        let diags = lint_with_max("foo(() => { bar(() => { baz(() => {}); }); });", 2);
        assert_eq!(
            diags.len(),
            1,
            "3 levels with max=2 should flag the innermost"
        );
    }

    #[test]
    fn test_allows_named_functions() {
        // Named function declarations don't count as callbacks
        let diags = lint_with_max("function a() { function b() { function c() {} } }", 1);
        assert!(
            diags.is_empty(),
            "named function declarations should not count as callbacks"
        );
    }

    #[test]
    fn test_flags_anonymous_functions() {
        let diags = lint_with_max(
            "foo(function() { bar(function() { baz(function() {}); }); });",
            2,
        );
        assert_eq!(
            diags.len(),
            1,
            "3 anonymous function levels with max=2 should flag"
        );
    }

    #[test]
    fn test_allows_single_callback() {
        let diags = lint_with_max("foo(() => {});", 1);
        assert!(diags.is_empty(), "single callback with max=1 should pass");
    }
}
