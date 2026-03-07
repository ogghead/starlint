//! Rule: `import/no-unassigned-import`
//!
//! Forbid unassigned (side-effect) imports like `import 'polyfill'`.
//! Side-effect imports make it hard to determine what a module depends on
//! and can cause unexpected behavior.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags side-effect imports that have no specifiers.
#[derive(Debug)]
pub struct NoUnassignedImport;

impl LintRule for NoUnassignedImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-unassigned-import".to_owned(),
            description: "Forbid unassigned (side-effect) imports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        // Side-effect import: `import 'foo'` — specifiers is empty
        // Empty named block: `import {} from 'foo'` — specifiers is empty
        let is_unassigned = import.specifiers.is_empty();

        if is_unassigned {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove side-effect import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-unassigned-import".to_owned(),
                message: "Unexpected side-effect import with no bindings".to_owned(),
                span: import_span,
                severity: Severity::Warning,
                help: None,
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnassignedImport)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_side_effect_import() {
        let diags = lint(r#"import "polyfill";"#);
        assert_eq!(diags.len(), 1, "side-effect import should be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "module";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_default_import() {
        let diags = lint(r#"import foo from "module";"#);
        assert!(diags.is_empty(), "default import should not be flagged");
    }
}
