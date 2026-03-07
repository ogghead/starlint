//! Rule: `typescript/no-this-alias`
//!
//! Disallow aliasing `this`. With arrow functions and `.bind()`, there is
//! rarely a need to assign `this` to a variable. The exception is
//! `const self = this`, which is a widely accepted convention.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags variable declarations that alias `this` (except `const self = this`).
#[derive(Debug)]
pub struct NoThisAlias;

impl LintRule for NoThisAlias {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-this-alias".to_owned(),
            description: "Disallow aliasing `this`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclarator])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclarator(decl) = node else {
            return;
        };

        // Must have an initializer that is `this`
        let Some(init_id) = decl.init else {
            return;
        };

        if !matches!(ctx.node(init_id), Some(AstNode::ThisExpression(_))) {
            return;
        }

        // Allow `const self = this` as a common acceptable pattern
        if let Some(name) = binding_name(decl.id, ctx) {
            if name == "self" {
                return;
            }
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/no-this-alias".to_owned(),
            message: "Do not alias `this` — use arrow functions or `.bind()` instead".to_owned(),
            span: Span::new(decl.span.start, decl.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Extract a simple identifier name from a binding pattern (resolved via `NodeId`).
///
/// Returns `None` for destructuring patterns (object, array, assignment).
fn binding_name<'a>(id: NodeId, ctx: &'a LintContext<'_>) -> Option<&'a str> {
    match ctx.node(id) {
        Some(AstNode::BindingIdentifier(ident)) => Some(ident.name.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoThisAlias)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_that_equals_this() {
        let diags = lint("const that = this;");
        assert_eq!(diags.len(), 1, "`const that = this` should be flagged");
    }

    #[test]
    fn test_flags_underscore_this() {
        let diags = lint("const _this = this;");
        assert_eq!(diags.len(), 1, "`const _this = this` should be flagged");
    }

    #[test]
    fn test_allows_self_equals_this() {
        let diags = lint("const self = this;");
        assert!(
            diags.is_empty(),
            "`const self = this` should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_assignment() {
        let diags = lint("const x = foo;");
        assert!(
            diags.is_empty(),
            "non-this assignment should not be flagged"
        );
    }
}
