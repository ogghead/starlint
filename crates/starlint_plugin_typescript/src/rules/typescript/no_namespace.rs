//! Rule: `typescript/no-namespace`
//!
//! Disallow `TypeScript` `namespace` and `module` declarations. Namespaces are
//! a legacy `TypeScript` feature that predates ES modules. Modern code should
//! use standard ES module imports/exports instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `namespace` and `module` declarations.
#[derive(Debug)]
pub struct NoNamespace;

impl LintRule for NoNamespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-namespace".to_owned(),
            description: "Disallow TypeScript `namespace` and `module` declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSModuleDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSModuleDeclaration(decl) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "typescript/no-namespace".to_owned(),
            message: "Do not use TypeScript namespaces — use ES modules instead".to_owned(),
            span: Span::new(decl.span.start, decl.span.end),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNamespace)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_namespace() {
        let diags = lint("namespace Foo { }");
        assert_eq!(diags.len(), 1, "`namespace` declaration should be flagged");
    }

    #[test]
    fn test_flags_module() {
        let diags = lint("module Foo { }");
        assert_eq!(diags.len(), 1, "`module` declaration should be flagged");
    }

    #[test]
    fn test_allows_regular_code() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "regular code should not be flagged");
    }
}
