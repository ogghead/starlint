//! Rule: `node/no-process-env`
//!
//! Disallow the use of `process.env`. Accessing environment variables
//! directly throughout a codebase makes configuration hard to track.
//! Prefer centralizing environment access in a config module.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `process.env` member expressions.
#[derive(Debug)]
pub struct NoProcessEnv;

impl LintRule for NoProcessEnv {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-process-env".to_owned(),
            description: "Disallow the use of `process.env`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticMemberExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticMemberExpression(member) = node else {
            return;
        };

        if member.property.as_str() != "env" {
            return;
        }

        let is_process = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "process"
        );

        if is_process {
            ctx.report(Diagnostic {
                rule_name: "node/no-process-env".to_owned(),
                message: "Unexpected use of `process.env` \u{2014} centralize environment access in a config module".to_owned(),
                span: Span::new(member.span.start, member.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoProcessEnv)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_process_env() {
        let diags = lint("const e = process.env;");
        assert_eq!(diags.len(), 1, "process.env should be flagged");
    }

    #[test]
    fn test_flags_process_env_property() {
        let diags = lint("var x = process.env.NODE_ENV;");
        assert_eq!(
            diags.len(),
            1,
            "process.env.NODE_ENV should flag the process.env part"
        );
    }

    #[test]
    fn test_allows_process_exit() {
        let diags = lint("process.exit(1);");
        assert!(diags.is_empty(), "process.exit should not be flagged");
    }

    #[test]
    fn test_allows_other_env() {
        let diags = lint("var x = env.foo;");
        assert!(
            diags.is_empty(),
            "env.foo without process should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_object_env() {
        let diags = lint("var x = config.env;");
        assert!(diags.is_empty(), "config.env should not be flagged");
    }
}
