//! Rule: `node/global-require`
//!
//! Disallow `require()` calls outside of the top-level module scope.
//! Calling `require()` inside functions, conditionals, or other nested
//! scopes makes dependency loading non-deterministic and harder to
//! statically analyze.

use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `require()` calls that are not at the top-level module scope.
///
/// Uses a depth counter to track whether the current node is nested
/// inside a function or arrow function expression.
#[derive(Debug)]
pub struct GlobalRequire {
    /// Current function nesting depth (0 = top-level).
    depth: RwLock<u32>,
}

impl GlobalRequire {
    /// Create a new `GlobalRequire` rule instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            depth: RwLock::new(0),
        }
    }
}

impl Default for GlobalRequire {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRule for GlobalRequire {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/global-require".to_owned(),
            description: "Disallow `require()` calls outside of the top-level module scope"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::CallExpression,
            AstNodeType::Function,
        ])
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::CallExpression,
            AstNodeType::Function,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::Function(_) | AstNode::ArrowFunctionExpression(_) => {
                if let Ok(mut guard) = self.depth.write() {
                    *guard = guard.saturating_add(1);
                }
            }
            AstNode::CallExpression(call) => {
                let is_require = matches!(
                    ctx.node(call.callee),
                    Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "require"
                );

                if !is_require {
                    return;
                }

                let inside_function = self.depth.read().ok().is_some_and(|guard| *guard > 0);

                if inside_function {
                    ctx.report(Diagnostic {
                        rule_name: "node/global-require".to_owned(),
                        message: "Unexpected `require()` inside a function \u{2014} move to top-level scope".to_owned(),
                        span: Span::new(call.span.start, call.span.end),
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
        if matches!(
            node,
            AstNode::Function(_) | AstNode::ArrowFunctionExpression(_)
        ) {
            if let Ok(mut guard) = self.depth.write() {
                *guard = guard.saturating_sub(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(GlobalRequire::new());

    #[test]
    fn test_flags_require_inside_function() {
        let diags = lint("function f() { require('x'); }");
        assert_eq!(diags.len(), 1, "require inside function should be flagged");
    }

    #[test]
    fn test_flags_require_inside_arrow() {
        let diags = lint("const f = () => { require('x'); };");
        assert_eq!(
            diags.len(),
            1,
            "require inside arrow function should be flagged"
        );
    }

    #[test]
    fn test_allows_top_level_require() {
        let diags = lint("require('x');");
        assert!(diags.is_empty(), "top-level require should not be flagged");
    }

    #[test]
    fn test_allows_top_level_const_require() {
        let diags = lint("const x = require('x');");
        assert!(
            diags.is_empty(),
            "top-level const require should not be flagged"
        );
    }

    #[test]
    fn test_flags_nested_function_require() {
        let diags = lint("function a() { function b() { require('x'); } }");
        assert_eq!(diags.len(), 1, "deeply nested require should be flagged");
    }
}
