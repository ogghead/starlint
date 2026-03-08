//! Rule: `class-methods-use-this`
//!
//! Flag class methods that don't reference `this`. Such methods could be
//! static or extracted to standalone functions, which makes intent clearer
//! and avoids unnecessary coupling to instances.

use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Tracking state for a method currently being traversed.
#[derive(Debug, Clone)]
struct MethodState {
    /// Span of the method definition (for reporting).
    span: Span,
    /// Whether a `this` expression was found inside this method.
    found_this: bool,
}

/// Flags non-static class methods that never reference `this`.
#[derive(Debug)]
pub struct ClassMethodsUseThis {
    /// Stack of method states; pushed on enter, popped on leave.
    stack: RwLock<Vec<MethodState>>,
}

impl ClassMethodsUseThis {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stack: RwLock::new(Vec::new()),
        }
    }
}

impl Default for ClassMethodsUseThis {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRule for ClassMethodsUseThis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "class-methods-use-this".to_owned(),
            description: "Require `this` in class methods".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition, AstNodeType::ThisExpression])
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition, AstNodeType::ThisExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, _ctx: &mut LintContext<'_>) {
        match node {
            AstNode::MethodDefinition(method) => {
                // Skip static methods — they can't use `this` on an instance.
                if method.is_static {
                    return;
                }

                // Skip constructors — they inherently use `this` for init.
                if method.kind == MethodDefinitionKind::Constructor {
                    return;
                }

                // Skip getters and setters — they inherently work with `this`.
                if method.kind == MethodDefinitionKind::Get
                    || method.kind == MethodDefinitionKind::Set
                {
                    return;
                }

                let Ok(mut guard) = self.stack.write() else {
                    return;
                };
                guard.push(MethodState {
                    span: Span::new(method.span.start, method.span.end),
                    found_this: false,
                });
            }
            AstNode::ThisExpression(_) => {
                // Mark the innermost method as having found `this`.
                let Ok(mut guard) = self.stack.write() else {
                    return;
                };
                if let Some(top) = guard.last_mut() {
                    top.found_this = true;
                }
            }
            _ => {}
        }
    }

    fn leave(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        // Only pop if we pushed (skip static, constructor, get, set).
        if method.is_static {
            return;
        }
        if method.kind == MethodDefinitionKind::Constructor
            || method.kind == MethodDefinitionKind::Get
            || method.kind == MethodDefinitionKind::Set
        {
            return;
        }

        let Ok(mut guard) = self.stack.write() else {
            return;
        };
        let Some(state) = guard.pop() else {
            return;
        };
        drop(guard);

        if !state.found_this {
            ctx.report(Diagnostic {
                rule_name: "class-methods-use-this".to_owned(),
                message: "Expected `this` to be used by class method".to_owned(),
                span: state.span,
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ClassMethodsUseThis::new())];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_method_without_this() {
        let diags = lint("class A { foo() { return 1; } }");
        assert_eq!(diags.len(), 1, "method without this should be flagged");
    }

    #[test]
    fn test_allows_method_with_this() {
        let diags = lint("class A { foo() { return this.x; } }");
        assert!(diags.is_empty(), "method using this should not be flagged");
    }

    #[test]
    fn test_allows_constructor() {
        let diags = lint("class A { constructor() { this.x = 1; } }");
        assert!(diags.is_empty(), "constructor should not be flagged");
    }

    #[test]
    fn test_allows_constructor_without_this() {
        let diags = lint("class A { constructor() { console.log('init'); } }");
        assert!(
            diags.is_empty(),
            "constructor without this should not be flagged (constructors are skipped)"
        );
    }

    #[test]
    fn test_allows_static_method() {
        let diags = lint("class A { static foo() { return 1; } }");
        assert!(diags.is_empty(), "static method should not be flagged");
    }

    #[test]
    fn test_allows_getter() {
        let diags = lint("class A { get x() { return this._x; } }");
        assert!(diags.is_empty(), "getter should not be flagged");
    }

    #[test]
    fn test_allows_setter() {
        let diags = lint("class A { set x(v) { this._x = v; } }");
        assert!(diags.is_empty(), "setter should not be flagged");
    }

    #[test]
    fn test_flags_multiple_methods_without_this() {
        let diags = lint("class A { foo() { return 1; } bar() { return 2; } }");
        assert_eq!(
            diags.len(),
            2,
            "both methods without this should be flagged"
        );
    }

    #[test]
    fn test_allows_method_with_nested_this() {
        let diags = lint("class A { foo() { if (true) { return this.x; } } }");
        assert!(
            diags.is_empty(),
            "method with nested this should not be flagged"
        );
    }
}
