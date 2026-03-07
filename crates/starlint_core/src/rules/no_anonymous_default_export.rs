//! Rule: `no-anonymous-default-export`
//!
//! Disallow anonymous default exports. Named exports improve discoverability
//! and make refactoring safer because tools can track references by name.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags anonymous default exports (functions, classes, and expressions).
#[derive(Debug)]
pub struct NoAnonymousDefaultExport;

impl LintRule for NoAnonymousDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-anonymous-default-export".to_owned(),
            description: "Disallow anonymous default exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExportDefaultDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ExportDefaultDeclaration(decl) = node else {
            return;
        };

        let is_anonymous = match ctx.node(decl.declaration) {
            // Named function/class declarations and expressions are fine
            Some(AstNode::Function(f)) => f.id.is_none(),
            Some(AstNode::Class(c)) => c.id.is_none(),
            // TS interfaces and identifier references are named
            Some(AstNode::TSInterfaceDeclaration(_) | AstNode::IdentifierReference(_)) => false,
            // Everything else (arrow functions, literals, objects, etc.) is anonymous
            _ => true,
        };

        if is_anonymous {
            ctx.report(Diagnostic {
                rule_name: "no-anonymous-default-export".to_owned(),
                message: "Assign a name to this default export for better discoverability"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
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
    fn test_flags_anonymous_function() {
        let diags = lint("export default function() {}");
        assert_eq!(
            diags.len(),
            1,
            "anonymous default function should be flagged"
        );
    }

    #[test]
    fn test_allows_named_function() {
        let diags = lint("export default function foo() {}");
        assert!(
            diags.is_empty(),
            "named default function should not be flagged"
        );
    }

    #[test]
    fn test_flags_anonymous_class() {
        let diags = lint("export default class {}");
        assert_eq!(diags.len(), 1, "anonymous default class should be flagged");
    }

    #[test]
    fn test_allows_named_class() {
        let diags = lint("export default class Foo {}");
        assert!(
            diags.is_empty(),
            "named default class should not be flagged"
        );
    }

    #[test]
    fn test_flags_arrow_function() {
        let diags = lint("export default () => {}");
        assert_eq!(
            diags.len(),
            1,
            "arrow function default export should be flagged"
        );
    }

    #[test]
    fn test_flags_literal_expression() {
        let diags = lint("export default 42");
        assert_eq!(diags.len(), 1, "literal default export should be flagged");
    }

    #[test]
    fn test_allows_identifier_reference() {
        let diags = lint("const foo = 42; export default foo;");
        assert!(
            diags.is_empty(),
            "identifier reference default export should not be flagged"
        );
    }
}
