//! Rule: `no-this-assignment` (unicorn)
//!
//! Disallow assigning `this` to a variable. With arrow functions and
//! `.bind()`, there's no need for `var self = this`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags `const self = this` and similar patterns.
#[derive(Debug)]
pub struct NoThisAssignment;

impl LintRule for NoThisAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-this-assignment".to_owned(),
            description: "Disallow assigning `this` to a variable".to_owned(),
            category: Category::Style,
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

        let Some(init_id) = decl.init else {
            return;
        };

        if matches!(ctx.node(init_id), Some(AstNode::ThisExpression(_))) {
            ctx.report(Diagnostic {
                rule_name: "no-this-assignment".to_owned(),
                message:
                    "Do not assign `this` to a variable — use arrow functions or `.bind()` instead"
                        .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoThisAssignment)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_this_assignment() {
        let diags = lint("const self = this;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_other_assignment() {
        let diags = lint("const x = 5;");
        assert!(diags.is_empty());
    }
}
