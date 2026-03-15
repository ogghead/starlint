//! Rule: `import/no-amd`
//!
//! Disallow AMD `require` and `define` calls. AMD module syntax is legacy
//! and should not be used in modern ES module or `CommonJS` codebases.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags AMD-style `define()` and `require()` calls with array dependencies.
#[derive(Debug)]
pub struct NoAmd;

impl LintRule for NoAmd {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-amd".to_owned(),
            description: "Disallow AMD require/define calls".to_owned(),
            category: Category::Suggestion,
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

        // Check if the callee is `define` or `require`
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
            _ => return,
        };

        if callee_name != "define" && callee_name != "require" {
            return;
        }

        // AMD pattern: first argument is an array of dependencies
        // e.g. define(['dep1', 'dep2'], function(dep1, dep2) { ... })
        // or   require(['dep1'], function(dep1) { ... })
        let first_arg = call.arguments.first();
        let has_array_arg = first_arg
            .is_some_and(|arg_id| matches!(ctx.node(*arg_id), Some(AstNode::ArrayExpression(_))));

        if has_array_arg {
            let callee_name_owned = callee_name.to_owned();
            let call_span = Span::new(call.span.start, call.span.end);
            ctx.report(Diagnostic {
                rule_name: "import/no-amd".to_owned(),
                message: format!(
                    "Expected imports instead of AMD '{callee_name_owned}' with dependency array"
                ),
                span: call_span,
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

    starlint_rule_framework::lint_rule_test!(NoAmd);

    #[test]
    fn test_flags_amd_define() {
        let diags = lint(r"define(['dep1', 'dep2'], function(dep1, dep2) {});");
        assert_eq!(
            diags.len(),
            1,
            "AMD define with dependency array should be flagged"
        );
    }

    #[test]
    fn test_flags_amd_require() {
        let diags = lint(r"require(['dep1'], function(dep1) {});");
        assert_eq!(
            diags.len(),
            1,
            "AMD require with dependency array should be flagged"
        );
    }

    #[test]
    fn test_allows_commonjs_require() {
        let diags = lint(r"const foo = require('foo');");
        assert!(
            diags.is_empty(),
            "CommonJS require without array should not be flagged"
        );
    }
}
