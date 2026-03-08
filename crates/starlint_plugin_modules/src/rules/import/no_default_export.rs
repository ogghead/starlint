//! Rule: `import/no-default-export`
//!
//! Disallow default exports. Named exports are preferable because they
//! enforce a consistent import name, improve refactoring tooling, and
//! make tree-shaking more effective.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags any `export default` declaration.
#[derive(Debug)]
pub struct NoDefaultExport;

impl LintRule for NoDefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-default-export".to_owned(),
            description: "Disallow default exports".to_owned(),
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

        // For named function/class declarations, suggest removing `default`.
        // `export default function foo()` → `export function foo()`
        // `export default class Foo` → `export class Foo`
        // Skip anonymous/expression exports since they can't become named exports.
        let decl_id = export.declaration;
        let fix = match ctx.node(decl_id) {
            Some(AstNode::Function(func)) if func.id.is_some() => {
                // Replace "export default " (15 chars) with "export "
                let kw_end = export.span.start.saturating_add(15);
                FixBuilder::new("Convert to named export", FixKind::SuggestionFix)
                    .replace(Span::new(export.span.start, kw_end), "export ")
                    .build()
            }
            Some(AstNode::Class(class)) if class.id.is_some() => {
                let kw_end = export.span.start.saturating_add(15);
                FixBuilder::new("Convert to named export", FixKind::SuggestionFix)
                    .replace(Span::new(export.span.start, kw_end), "export ")
                    .build()
            }
            _ => None,
        };

        ctx.report(Diagnostic {
            rule_name: "import/no-default-export".to_owned(),
            message: "Prefer named exports over default exports".to_owned(),
            span: Span::new(export.span.start, export.span.end),
            severity: Severity::Warning,
            help: Some("Use a named export instead".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDefaultExport)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_default_export_function() {
        let diags = lint("export default function foo() {}");
        assert_eq!(diags.len(), 1, "default export function should be flagged");
    }

    #[test]
    fn test_flags_default_export_value() {
        let diags = lint("export default 42;");
        assert_eq!(diags.len(), 1, "default export value should be flagged");
    }

    #[test]
    fn test_allows_named_export() {
        let diags = lint("export const foo = 42;");
        assert!(diags.is_empty(), "named export should not be flagged");
    }
}
