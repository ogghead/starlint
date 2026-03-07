//! Rule: `typescript/no-require-imports`
//!
//! Disallow `require()` calls entirely. In `TypeScript` projects, `require()`
//! bypasses the module type system. Use `import` declarations instead, which
//! are statically analyzed and provide better tooling support.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags any `require()` call expression.
#[derive(Debug)]
pub struct NoRequireImports;

impl LintRule for NoRequireImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-require-imports".to_owned(),
            description: "Disallow `require()` calls".to_owned(),
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

        let is_require = matches!(
            ctx.node(call.callee),
            Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "require"
        );

        if !is_require {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/no-require-imports".to_owned(),
            message: "Use `import` instead of `require()`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRequireImports)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_bare_require() {
        let diags = lint("require(\"foo\");");
        assert_eq!(diags.len(), 1, "bare `require()` call should be flagged");
    }

    #[test]
    fn test_flags_require_in_variable() {
        let diags = lint("const x = require(\"bar\");");
        assert_eq!(
            diags.len(),
            1,
            "`require()` in variable init should be flagged"
        );
    }

    #[test]
    fn test_allows_import() {
        let diags = lint("import x from \"foo\";");
        assert!(
            diags.is_empty(),
            "`import` declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_require_call() {
        let diags = lint("foo();");
        assert!(diags.is_empty(), "non-`require` call should not be flagged");
    }

    #[test]
    fn test_allows_method_named_require() {
        let diags = lint("obj.require(\"foo\");");
        assert!(
            diags.is_empty(),
            "method call named `require` should not be flagged"
        );
    }
}
