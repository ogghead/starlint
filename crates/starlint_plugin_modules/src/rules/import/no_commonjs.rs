//! Rule: `import/no-commonjs`
//!
//! Disallow `CommonJS` `require()` calls and `module.exports` / `exports`
//! assignments. Encourages use of ES module syntax instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `CommonJS` `require()` calls and `module.exports` usage.
#[derive(Debug)]
pub struct NoCommonjs;

impl LintRule for NoCommonjs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-commonjs".to_owned(),
            description: "Disallow CommonJS require/module.exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::AssignmentExpression,
            AstNodeType::CallExpression,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::CallExpression(call) => {
                // Check for require('...')
                let is_require = matches!(
                    ctx.node(call.callee),
                    Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "require"
                );

                if is_require {
                    // Only flag if the first argument is a string literal (standard require)
                    let has_string_arg = call
                        .arguments
                        .first()
                        .and_then(|arg_id| ctx.node(*arg_id))
                        .is_some_and(|arg| matches!(arg, AstNode::StringLiteral(_)));

                    if has_string_arg {
                        ctx.report(Diagnostic {
                            rule_name: "import/no-commonjs".to_owned(),
                            message: "Use ES module import instead of CommonJS require()"
                                .to_owned(),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstNode::AssignmentExpression(assign) => {
                // Check for module.exports = ... or exports.foo = ...
                let is_module_exports = is_module_exports_member(assign.left, ctx);

                if is_module_exports {
                    ctx.report(Diagnostic {
                        rule_name: "import/no-commonjs".to_owned(),
                        message: "Use ES module export instead of CommonJS module.exports"
                            .to_owned(),
                        span: Span::new(assign.span.start, assign.span.end),
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
}

/// Check if a member expression is `module.exports` or `exports.<name>`.
fn is_module_exports_member(member_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(member_id) else {
        return false;
    };
    let prop_name = member.property.as_str();

    match ctx.node(member.object) {
        Some(AstNode::IdentifierReference(id)) => {
            let obj_name = id.name.as_str();
            // module.exports = ...
            (obj_name == "module" && prop_name == "exports")
            // exports.foo = ...
            || obj_name == "exports"
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCommonjs)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_require_call() {
        let diags = lint(r"const foo = require('foo');");
        assert_eq!(diags.len(), 1, "CommonJS require should be flagged");
    }

    #[test]
    fn test_flags_module_exports() {
        let diags = lint("module.exports = {};");
        assert_eq!(
            diags.len(),
            1,
            "module.exports assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_es_import() {
        let diags = lint(r#"import foo from "foo";"#);
        assert!(diags.is_empty(), "ES import should not be flagged");
    }
}
