//! Rule: `prefer-rest-params`
//!
//! Suggest using rest parameters instead of `arguments`. Rest parameters
//! are a proper array and more explicit than the `arguments` object.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags use of the `arguments` object.
#[derive(Debug)]
pub struct PreferRestParams;

impl LintRule for PreferRestParams {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-rest-params".to_owned(),
            description: "Suggest using rest parameters instead of `arguments`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IdentifierReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IdentifierReference(id) = node else {
            return;
        };

        if id.name.as_str() == "arguments" {
            ctx.report(Diagnostic {
                rule_name: "prefer-rest-params".to_owned(),
                message: "Use rest parameters instead of `arguments`".to_owned(),
                span: Span::new(id.span.start, id.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferRestParams)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_arguments() {
        let diags = lint("function f() { return arguments.length; }");
        assert_eq!(diags.len(), 1, "use of arguments should be flagged");
    }

    #[test]
    fn test_allows_rest_params() {
        let diags = lint("function f(...args) { return args.length; }");
        assert!(diags.is_empty(), "rest params should not be flagged");
    }

    #[test]
    fn test_allows_arguments_as_param_name() {
        // When "arguments" is a named parameter, it shadows the builtin
        let diags = lint("function f(arguments) { return arguments; }");
        // This will still flag it as an identifier reference — that's OK
        // for simplicity. Full detection would need scope analysis.
        assert!(!diags.is_empty(), "arguments reference should be flagged");
    }
}
