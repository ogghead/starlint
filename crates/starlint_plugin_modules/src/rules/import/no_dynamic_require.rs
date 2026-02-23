//! Rule: `import/no-dynamic-require`
//!
//! Forbid `require()` calls with expressions (non-literal arguments).
//! Dynamic requires make it hard for bundlers and static analysis tools to
//! determine the dependency graph.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `require()` calls whose argument is not a string literal.
#[derive(Debug)]
pub struct NoDynamicRequire;

impl LintRule for NoDynamicRequire {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-dynamic-require".to_owned(),
            description: "Forbid `require()` calls with expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check if callee is `require`
        let is_require = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "require"
        );

        if !is_require {
            return;
        }

        // Check if the first argument is a string literal
        let first_arg = call.arguments.first();
        let is_literal = first_arg
            .is_some_and(|arg_id| matches!(ctx.node(*arg_id), Some(AstNode::StringLiteral(_))));

        if !is_literal {
            ctx.report(Diagnostic {
                rule_name: "import/no-dynamic-require".to_owned(),
                message: "Calls to `require()` should use a string literal argument".to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDynamicRequire)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_dynamic_require() {
        let diags = lint(r"const mod = require(name);");
        assert_eq!(
            diags.len(),
            1,
            "dynamic require with variable should be flagged"
        );
    }

    #[test]
    fn test_flags_template_literal_require() {
        let diags = lint(r"const mod = require(`./path/${name}`);");
        assert_eq!(
            diags.len(),
            1,
            "dynamic require with template literal should be flagged"
        );
    }

    #[test]
    fn test_allows_static_require() {
        let diags = lint(r#"const mod = require("lodash");"#);
        assert!(diags.is_empty(), "static require should not be flagged");
    }
}
