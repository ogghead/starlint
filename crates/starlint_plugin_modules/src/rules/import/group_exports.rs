//! Rule: `import/group-exports`
//!
//! Prefer use of a single export declaration rather than scattered exports
//! throughout the file. This makes it easier to see what a module provides
//! at a glance.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags modules with multiple named export declarations that could be grouped.
#[derive(Debug)]
pub struct GroupExports;

impl LintRule for GroupExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/group-exports".to_owned(),
            description: "Prefer a single export declaration rather than scattered exports"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Program])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Program(program) = node else {
            return;
        };

        // Collect spans of all named export declarations (excluding re-exports)
        let named_export_spans: Vec<Span> = program
            .body
            .iter()
            .filter_map(|&stmt_id| {
                if let Some(AstNode::ExportNamedDeclaration(export)) = ctx.node(stmt_id) {
                    // Only count local exports, not re-exports like `export { x } from 'y'`
                    if export.source.is_none() {
                        return Some(Span::new(export.span.start, export.span.end));
                    }
                }
                None
            })
            .collect();

        // If there are more than one named export declaration, flag all but the first
        if named_export_spans.len() > 1 {
            for &span in named_export_spans.iter().skip(1) {
                ctx.report(Diagnostic {
                    rule_name: "import/group-exports".to_owned(),
                    message: "Multiple named export declarations; prefer a single export { ... }"
                        .to_owned(),
                    span,
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(GroupExports)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_scattered_exports() {
        let source = "export const a = 1;\nexport const b = 2;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "scattered named exports should be flagged");
    }

    #[test]
    fn test_allows_single_export() {
        let source = "const a = 1;\nconst b = 2;\nexport { a, b };";
        let diags = lint(source);
        assert!(diags.is_empty(), "single grouped export should be fine");
    }

    #[test]
    fn test_allows_re_exports() {
        let source = "export { foo } from 'foo';\nexport { bar } from 'bar';";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "re-exports from different modules should not be flagged"
        );
    }
}
