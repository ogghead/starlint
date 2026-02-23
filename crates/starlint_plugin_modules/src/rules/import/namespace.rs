//! Rule: `import/namespace`
//!
//! Validate namespace (star) imports. Namespace imports pull in everything
//! from a module and can mask unused dependencies. This rule flags namespace
//! imports from JSON modules (which only have a default export) as a
//! heuristic without full module resolution.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags potentially invalid namespace imports.
#[derive(Debug)]
pub struct NamespaceImport;

impl LintRule for NamespaceImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/namespace".to_owned(),
            description: "Validate namespace (star) imports".to_owned(),
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

        // Type-only imports don't need runtime validation
        if import.import_kind_is_type {
            return;
        }

        let specifiers = &import.specifiers;
        // In starlint_ast, namespace imports are represented as ImportSpecifier
        // with imported == "*". Check if any specifier is a namespace import.
        let has_namespace = specifiers.iter().any(|spec_id| {
            ctx.node(*spec_id)
                .is_some_and(|n| matches!(n, AstNode::ImportSpecifier(s) if s.imported == "*"))
        });

        if !has_namespace {
            return;
        }

        let source_value = import.source.as_str();

        // Heuristic: namespace import from JSON makes no sense
        if std::path::Path::new(source_value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            ctx.report(Diagnostic {
                rule_name: "import/namespace".to_owned(),
                message: "Namespace import from JSON module is not useful (JSON modules only have a default export)".to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NamespaceImport)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_namespace_import_from_json() {
        let diags = lint(r#"import * as data from "./data.json";"#);
        assert_eq!(
            diags.len(),
            1,
            "namespace import from JSON should be flagged"
        );
    }

    #[test]
    fn test_allows_namespace_import_from_js() {
        let diags = lint(r#"import * as utils from "./utils";"#);
        assert!(
            diags.is_empty(),
            "namespace import from JS module should not be flagged"
        );
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "./module";"#);
        assert!(
            diags.is_empty(),
            "named import should not be flagged by namespace rule"
        );
    }
}
