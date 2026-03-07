//! Rule: `import/no-relative-parent-imports`
//!
//! Forbid importing from parent directories (`../`). Parent imports can
//! create tightly-coupled code and make refactoring harder.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags imports whose source begins with `../`.
#[derive(Debug)]
pub struct NoRelativeParentImports;

impl LintRule for NoRelativeParentImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-relative-parent-imports".to_owned(),
            description: "Forbid importing from parent directories".to_owned(),
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

        let source_value = import.source.as_str();

        if source_value.starts_with("../") || source_value == ".." {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove parent directory import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-relative-parent-imports".to_owned(),
                message: "Relative parent imports are not allowed".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoRelativeParentImports)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_parent_import() {
        let diags = lint(r#"import foo from "../utils";"#);
        assert_eq!(diags.len(), 1, "parent directory import should be flagged");
    }

    #[test]
    fn test_flags_deep_parent_import() {
        let diags = lint(r#"import bar from "../../lib/helpers";"#);
        assert_eq!(
            diags.len(),
            1,
            "deep parent directory import should be flagged"
        );
    }

    #[test]
    fn test_allows_sibling_import() {
        let diags = lint(r#"import baz from "./sibling";"#);
        assert!(
            diags.is_empty(),
            "sibling directory import should not be flagged"
        );
    }
}
