//! Rule: `import/named`
//!
//! Validate that named imports correspond to named exports in the resolved
//! module. Without full module resolution this rule performs a heuristic
//! check: it flags named imports from relative paths ending in `.json`
//! since JSON modules only expose a default export.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags named imports that are unlikely to exist in the resolved module.
#[derive(Debug)]
pub struct NamedExport;

impl LintRule for NamedExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/named".to_owned(),
            description:
                "Validate that named imports correspond to named exports in the resolved module"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        // Type-only imports don't need runtime exports
        if import.import_kind_is_type {
            return;
        }

        let source_value = import.source.as_str();

        // Heuristic: JSON modules only have a default export
        if !std::path::Path::new(source_value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            return;
        }

        let specifiers = &import.specifiers;
        for spec_id in specifiers {
            if let Some(AstNode::ImportSpecifier(named)) = ctx.node(*spec_id) {
                // Skip default imports (imported == "default")
                if named.imported == "default" {
                    continue;
                }
                ctx.report(Diagnostic {
                    rule_name: "import/named".to_owned(),
                    message: format!(
                        "'{}' is not exported from '{}' (JSON modules only have a default export)",
                        named.local.as_str(),
                        source_value,
                    ),
                    span: Span::new(named.span.start, named.span.end),
                    severity: Severity::Error,
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NamedExport)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_named_import_from_json() {
        let diags = lint(r#"import { foo } from "./data.json";"#);
        assert_eq!(
            diags.len(),
            1,
            "named import from JSON module should be flagged"
        );
    }

    #[test]
    fn test_allows_default_import_from_json() {
        let diags = lint(r#"import data from "./data.json";"#);
        assert!(
            diags.is_empty(),
            "default import from JSON should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_named_import_from_js() {
        let diags = lint(r#"import { foo } from "./module";"#);
        assert!(
            diags.is_empty(),
            "named import from JS module should not be flagged"
        );
    }
}
