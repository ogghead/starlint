//! Rule: `prefer-module`
//!
//! Prefer ESM (`import`/`export`) over `CommonJS` (`require`/`module.exports`).
//! Flags `require()` calls with a string argument, `module.exports = ...`,
//! and `exports.foo = ...` assignments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node::{AssignmentExpressionNode, CallExpressionNode};
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `CommonJS` patterns in favor of ESM `import`/`export`.
#[derive(Debug)]
pub struct PreferModule;

impl LintRule for PreferModule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-module".to_owned(),
            description: "Prefer ESM `import`/`export` over CommonJS `require`/`module.exports`"
                .to_owned(),
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
            AstNode::CallExpression(call) => check_require(call, ctx),
            AstNode::AssignmentExpression(assign) => check_exports_assign(assign, ctx),
            _ => {}
        }
    }
}

/// Check for `require('...')` calls with a string literal argument.
fn check_require(call: &CallExpressionNode, ctx: &mut LintContext<'_>) {
    let Some(AstNode::IdentifierReference(callee_id)) = ctx.node(call.callee) else {
        return;
    };

    if callee_id.name.as_str() != "require" {
        return;
    }

    // Only flag require() with a single string argument.
    let has_string_arg = call
        .arguments
        .first()
        .is_some_and(|arg_id| matches!(ctx.node(*arg_id), Some(AstNode::StringLiteral(_))));

    if has_string_arg {
        ctx.report(Diagnostic {
            rule_name: "prefer-module".to_owned(),
            message: "Use ESM `import` instead of `require()`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check for `module.exports = ...` and `exports.foo = ...` assignments.
fn check_exports_assign(assign: &AssignmentExpressionNode, ctx: &mut LintContext<'_>) {
    // assign.left is NodeId — resolve to check for StaticMemberExpression
    let is_commonjs_export = match ctx.node(assign.left) {
        // `module.exports = ...` or `exports.foo = ...`
        Some(AstNode::StaticMemberExpression(member)) => {
            is_module_exports_target(member, ctx) || is_exports_property_target(member, ctx)
        }
        _ => false,
    };

    if is_commonjs_export {
        ctx.report(Diagnostic {
            rule_name: "prefer-module".to_owned(),
            message: "Use ESM `export` instead of `CommonJS` `module.exports` / `exports`"
                .to_owned(),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check if a static member expression target is `module.exports`.
fn is_module_exports_target(
    member: &starlint_ast::node::StaticMemberExpressionNode,
    ctx: &LintContext<'_>,
) -> bool {
    member.property.as_str() == "exports"
        && matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "module"
        )
}

/// Check if a static member expression target is `exports.foo`.
fn is_exports_property_target(
    member: &starlint_ast::node::StaticMemberExpressionNode,
    ctx: &LintContext<'_>,
) -> bool {
    matches!(
        ctx.node(member.object),
        Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "exports"
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferModule);

    #[test]
    fn test_flags_require() {
        let diags = lint("const x = require('foo');");
        assert_eq!(diags.len(), 1, "require() should be flagged");
    }

    #[test]
    fn test_flags_module_exports() {
        let diags = lint("module.exports = {};");
        assert_eq!(diags.len(), 1, "module.exports should be flagged");
    }

    #[test]
    fn test_flags_exports_property() {
        let diags = lint("exports.foo = bar;");
        assert_eq!(diags.len(), 1, "exports.foo should be flagged");
    }

    #[test]
    fn test_allows_esm_import() {
        let diags = lint("import x from 'foo';");
        assert!(diags.is_empty(), "ESM import should not be flagged");
    }

    #[test]
    fn test_allows_esm_export() {
        let diags = lint("export default {};");
        assert!(diags.is_empty(), "ESM export should not be flagged");
    }

    #[test]
    fn test_allows_require_without_string_arg() {
        let diags = lint("require(variable);");
        assert!(
            diags.is_empty(),
            "require() with non-string argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_assignment() {
        let diags = lint("foo.bar = 1;");
        assert!(
            diags.is_empty(),
            "unrelated assignment should not be flagged"
        );
    }
}
