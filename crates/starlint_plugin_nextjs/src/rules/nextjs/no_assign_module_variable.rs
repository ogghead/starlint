//! Rule: `nextjs/no-assign-module-variable`
//!
//! Forbid assigning to the `module` variable, which interferes with
//! Next.js module handling and hot module replacement.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-assign-module-variable";

/// Flags assignment expressions where the left side is the `module` variable.
#[derive(Debug)]
pub struct NoAssignModuleVariable;

impl LintRule for NoAssignModuleVariable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid assigning to the `module` variable".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        let is_module_target = matches!(
            ctx.node(assign.left),
            Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "module"
        );

        if is_module_target {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not assign to the `module` variable -- it interferes with Next.js module handling".to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Error,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAssignModuleVariable)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_module_assignment() {
        let diags = lint("module = { exports: {} };");
        assert_eq!(diags.len(), 1, "module assignment should be flagged");
    }

    #[test]
    fn test_allows_module_exports() {
        let diags = lint("module.exports = {};");
        assert!(diags.is_empty(), "module.exports should not be flagged");
    }

    #[test]
    fn test_allows_other_variable() {
        let diags = lint("let x = 1;");
        assert!(
            diags.is_empty(),
            "other variable assignment should not be flagged"
        );
    }
}
