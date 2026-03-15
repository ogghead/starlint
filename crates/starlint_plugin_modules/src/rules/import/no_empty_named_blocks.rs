//! Rule: `import/no-empty-named-blocks`
//!
//! Forbid empty named import blocks (`import {} from 'mod'`).
//! An empty import block is likely a mistake or leftover from refactoring.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags import declarations with empty named import blocks.
#[derive(Debug)]
pub struct NoEmptyNamedBlocks;

impl LintRule for NoEmptyNamedBlocks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-empty-named-blocks".to_owned(),
            description: "Forbid empty named import blocks".to_owned(),
            category: Category::Correctness,
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

        // Empty named block: the specifiers list exists but is empty
        // This catches `import {} from 'mod'`
        // Side-effect imports (`import 'mod'`) also have an empty specifiers
        // list, so we need to check the source text to distinguish.
        if import.specifiers.is_empty() {
            // Side-effect imports don't have `{` in the source text before `from`
            let src = ctx.source_text();
            let start = usize::try_from(import.span.start).unwrap_or(0);
            let end = usize::try_from(import.span.end).unwrap_or(0);
            let import_text = src.get(start..end).unwrap_or("");
            if !import_text.contains('{') {
                return;
            }
        }

        if import.specifiers.is_empty() {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove the empty import statement", FixKind::SafeFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-empty-named-blocks".to_owned(),
                message: "Unexpected empty named import block".to_owned(),
                span: import_span,
                severity: Severity::Warning,
                help: Some("Remove the empty import statement".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoEmptyNamedBlocks);

    #[test]
    fn test_flags_empty_named_block() {
        let diags = lint(r#"import {} from "mod";"#);
        assert_eq!(diags.len(), 1, "empty named import block should be flagged");
    }

    #[test]
    fn test_allows_named_imports() {
        let diags = lint(r#"import { foo } from "mod";"#);
        assert!(
            diags.is_empty(),
            "non-empty named import block should not be flagged"
        );
    }

    #[test]
    fn test_allows_side_effect_import() {
        let diags = lint(r#"import "mod";"#);
        assert!(diags.is_empty(), "side-effect import should not be flagged");
    }
}
