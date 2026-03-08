//! Rule: `consistent-destructuring` (unicorn)
//!
//! Flags member expression access on objects that have already been
//! destructured. If `obj` was destructured via `const { a } = obj;`, then
//! accessing `obj.b` should instead add `b` to the destructuring pattern.
//!
//! This is a simplified implementation that tracks destructured object names
//! within block scopes using a stack-based approach.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use std::collections::HashSet;
use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Tracks destructured object names per block scope.
#[derive(Debug)]
pub struct ConsistentDestructuring {
    /// Stack of sets of destructured object names, one per scope level.
    destructured_names: RwLock<Vec<HashSet<String>>>,
}

impl ConsistentDestructuring {
    /// Create a new `ConsistentDestructuring` rule.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            destructured_names: RwLock::new(Vec::new()),
        }
    }
}

impl Default for ConsistentDestructuring {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the identifier name from a node if it is a simple identifier.
fn node_identifier_name(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::IdentifierReference(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

impl LintRule for ConsistentDestructuring {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-destructuring".to_owned(),
            description:
                "Use destructured variables instead of member expressions on destructured objects"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::BlockStatement,
            AstNodeType::Function,
            AstNodeType::Program,
            AstNodeType::StaticMemberExpression,
            AstNodeType::VariableDeclarator,
        ])
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::BlockStatement,
            AstNodeType::Function,
            AstNodeType::Program,
            AstNodeType::StaticMemberExpression,
            AstNodeType::VariableDeclarator,
        ])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            // Track scope entry via block statements, function bodies, and program
            AstNode::BlockStatement(_) | AstNode::Program(_) | AstNode::Function(_) => {
                if let Ok(mut stack) = self.destructured_names.write() {
                    stack.push(HashSet::new());
                }
            }

            // When we see a variable declarator with an object pattern, record the
            // name of the object being destructured.
            AstNode::VariableDeclarator(decl) => {
                // Resolve the binding to check if it's an ObjectPattern
                if let Some(AstNode::ObjectPattern(obj_pat)) = ctx.node(decl.id) {
                    // Only track if there are actual destructured properties
                    if !obj_pat.properties.is_empty() {
                        if let Some(init_id) = decl.init {
                            if let Some(init_node) = ctx.node(init_id) {
                                if let Some(name) = node_identifier_name(init_node) {
                                    if let Ok(mut stack) = self.destructured_names.write() {
                                        if let Some(scope) = stack.last_mut() {
                                            scope.insert(name.to_owned());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check static member expressions (e.g. `obj.b`) against tracked names.
            AstNode::StaticMemberExpression(member) => {
                let obj_name = ctx.node(member.object).and_then(node_identifier_name);
                if let Some(obj_name) = obj_name {
                    let is_destructured =
                        self.destructured_names.read().ok().is_some_and(|stack| {
                            stack.iter().any(|scope| scope.contains(obj_name))
                        });

                    if is_destructured {
                        let prop_name = member.property.as_str();
                        ctx.report(Diagnostic {
                            rule_name: "consistent-destructuring".to_owned(),
                            message: format!(
                                "Use destructuring for `{obj_name}.{prop_name}` instead of \
                                 accessing a property on an already-destructured object"
                            ),
                            span: Span::new(member.span.start, member.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }

            _ => {}
        }
    }

    #[allow(clippy::shadow_unrelated)]
    fn leave(&self, _node_id: NodeId, node: &AstNode, _ctx: &mut LintContext<'_>) {
        // Pop scope when leaving block/program/function
        if matches!(
            node,
            AstNode::BlockStatement(_) | AstNode::Program(_) | AstNode::Function(_)
        ) {
            if let Ok(mut stack) = self.destructured_names.write() {
                let _popped = stack.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentDestructuring::new())];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_member_access_on_destructured_object() {
        let diags = lint("const { a } = obj;\nobj.b;");
        assert_eq!(
            diags.len(),
            1,
            "accessing obj.b after destructuring obj should be flagged"
        );
        assert!(
            diags.first().is_some_and(|d| d.message.contains("obj.b")),
            "should mention the member expression"
        );
    }

    #[test]
    fn test_allows_full_destructuring() {
        let diags = lint("const { a, b } = obj;");
        assert!(
            diags.is_empty(),
            "complete destructuring should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_destructured_member_access() {
        let diags = lint("foo.bar;");
        assert!(
            diags.is_empty(),
            "member access on non-destructured object should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_object() {
        let diags = lint("const { a } = obj;\nother.b;");
        assert!(
            diags.is_empty(),
            "member access on a different object should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_accesses() {
        let diags = lint("const { a } = obj;\nobj.b;\nobj.c;");
        assert_eq!(
            diags.len(),
            2,
            "multiple member accesses on destructured object should each be flagged"
        );
    }

    #[test]
    fn test_flags_in_variable_init() {
        let diags = lint("const { a } = obj;\nconst x = obj.b;");
        assert_eq!(
            diags.len(),
            1,
            "member access in variable init after destructuring should be flagged"
        );
    }

    #[test]
    fn test_allows_no_properties_destructured() {
        // Edge case: empty destructuring pattern (no properties)
        let diags = lint("const {} = obj;\nobj.b;");
        assert!(
            diags.is_empty(),
            "empty destructuring should not trigger the rule"
        );
    }

    #[test]
    fn test_allows_normal_variable_declaration() {
        let diags = lint("const x = 1;\nobj.foo;");
        assert!(
            diags.is_empty(),
            "non-destructuring declaration should not affect member access checks"
        );
    }
}
