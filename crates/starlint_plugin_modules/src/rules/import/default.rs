//! Rule: `import/default`
//!
//! Ensure a default export is present when a default import is used.
//! This is a static analysis approximation — it checks whether the import
//! declaration has a default specifier, which can be paired with module
//! resolution in the future.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags default imports from modules that may not have a default export.
///
/// Without full module resolution this rule flags default imports from
/// obviously-named-only modules (heuristic: source ending in `/index`).
#[derive(Debug)]
pub struct DefaultExport;

impl LintRule for DefaultExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/default".to_owned(),
            description: "Ensure a default export is present when a default import is used"
                .to_owned(),
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

        let specifiers = &import.specifiers;
        // Check if any specifier resolves to a default import
        // In starlint_ast, specifiers are NodeIds that resolve to ImportSpecifier nodes.
        // A default import has imported == "default".
        let has_default = specifiers.iter().any(|spec_id| {
            ctx.node(*spec_id).is_some_and(
                |n| matches!(n, AstNode::ImportSpecifier(s) if s.imported == "default"),
            )
        });

        if !has_default {
            return;
        }

        // Type-only imports don't need runtime default exports
        if import.import_kind_is_type {
            return;
        }

        let source_value = import.source.as_str();

        // Heuristic: flag default imports from JSON files (they have no default export
        // in strict ESM) — this is a common mistake
        if std::path::Path::new(source_value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            ctx.report(Diagnostic {
                rule_name: "import/default".to_owned(),
                message: "No default export found in imported JSON module".to_owned(),
                span: Span::new(import.span.start, import.span.end),
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

    starlint_rule_framework::lint_rule_test!(DefaultExport);

    #[test]
    fn test_flags_default_import_from_json() {
        let diags = lint(r#"import data from "./data.json";"#);
        assert_eq!(diags.len(), 1, "default import from JSON should be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "./module";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_default_import_from_js() {
        let diags = lint(r#"import foo from "./module";"#);
        assert!(
            diags.is_empty(),
            "default import from JS module should not be flagged"
        );
    }
}
