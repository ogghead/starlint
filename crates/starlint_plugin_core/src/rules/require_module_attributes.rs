//! Rule: `require-module-attributes`
//!
//! Flag import declarations for JSON, CSS, or WASM modules that are missing
//! import attributes (also known as import assertions). Non-JS modules
//! should use `with { type: '...' }` to declare their type.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// File extensions that require import attributes.
const EXTENSIONS_NEEDING_ATTRIBUTES: &[&str] = &[".json", ".css", ".wasm"];

/// Flags non-JS module imports that are missing import attributes.
#[derive(Debug)]
pub struct RequireModuleAttributes;

impl LintRule for RequireModuleAttributes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-module-attributes".to_owned(),
            description: "Require import attributes for non-JS modules".to_owned(),
            category: Category::Suggestion,
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

        let source_str = import.source.as_str();

        // Check if the import source ends with a non-JS extension
        let needs_attributes = EXTENSIONS_NEEDING_ATTRIBUTES
            .iter()
            .any(|ext| source_str.ends_with(ext));

        if !needs_attributes {
            return;
        }

        // Check if import has a `with` clause by inspecting the source text.
        // The flat AST does not preserve the `with` clause, so we check
        // the source text for `with {` or `assert {` after the source string.
        let src = ctx.source_text();
        let import_start = usize::try_from(import.span.start).unwrap_or(0);
        let import_end = usize::try_from(import.span.end).unwrap_or(0);
        let import_text = src.get(import_start..import_end).unwrap_or("");
        let has_attributes = import_text.contains(" with ") || import_text.contains(" assert ");

        if !has_attributes {
            ctx.report(Diagnostic {
                rule_name: "require-module-attributes".to_owned(),
                message: format!("Import from '{source_str}' is missing import attributes"),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireModuleAttributes)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_json_import_without_attributes() {
        let diags = lint("import data from './data.json';");
        assert_eq!(
            diags.len(),
            1,
            "JSON import without attributes should be flagged"
        );
    }

    #[test]
    fn test_allows_json_import_with_attributes() {
        let diags = lint("import data from './data.json' with { type: 'json' };");
        assert!(
            diags.is_empty(),
            "JSON import with attributes should not be flagged"
        );
    }

    #[test]
    fn test_allows_js_import_without_attributes() {
        let diags = lint("import foo from './foo.js';");
        assert!(
            diags.is_empty(),
            "JS import without attributes should not be flagged"
        );
    }

    #[test]
    fn test_flags_css_import_without_attributes() {
        let diags = lint("import styles from './styles.css';");
        assert_eq!(
            diags.len(),
            1,
            "CSS import without attributes should be flagged"
        );
    }

    #[test]
    fn test_flags_wasm_import_without_attributes() {
        let diags = lint("import mod from './module.wasm';");
        assert_eq!(
            diags.len(),
            1,
            "WASM import without attributes should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_module_import() {
        let diags = lint("import foo from 'lodash';");
        assert!(diags.is_empty(), "bare module import should not be flagged");
    }
}
