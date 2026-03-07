//! Rule: `import/no-anonymous-default-export`
//!
//! Disallow anonymous default exports. Named default exports improve
//! stack traces, refactoring tools, and make the module's API clearer.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags anonymous default export declarations.
#[derive(Debug)]
pub struct NoAnonymousDefaultExport;

impl LintRule for NoAnonymousDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-anonymous-default-export".to_owned(),
            description: "Disallow anonymous default exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExportDefaultDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ExportDefaultDeclaration(export) = node else {
            return;
        };

        let is_anonymous = match ctx.node(export.declaration) {
            Some(AstNode::Function(f)) => f.id.is_none(),
            Some(AstNode::Class(c)) => c.id.is_none(),
            // TS interfaces and identifier references are named
            Some(AstNode::TSInterfaceDeclaration(_) | AstNode::IdentifierReference(_)) => false,
            // Everything else (arrow functions, literals, objects, etc.) is anonymous
            _ => true,
        };

        if is_anonymous {
            ctx.report(Diagnostic {
                rule_name: "import/no-anonymous-default-export".to_owned(),
                message: "Assign a name to the default export for better debugging and refactoring"
                    .to_owned(),
                span: Span::new(export.span.start, export.span.end),
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAnonymousDefaultExport)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_anonymous_arrow_function() {
        let diags = lint("export default () => {};");
        assert_eq!(
            diags.len(),
            1,
            "anonymous arrow function default export should be flagged"
        );
    }

    #[test]
    fn test_flags_anonymous_object() {
        let diags = lint("export default {};");
        assert_eq!(
            diags.len(),
            1,
            "anonymous object default export should be flagged"
        );
    }

    #[test]
    fn test_allows_named_function() {
        let diags = lint("export default function myFunc() {}");
        assert!(
            diags.is_empty(),
            "named function default export should not be flagged"
        );
    }
}
