//! Rule: `require-module-specifiers`
//!
//! Flag import declarations that have no specifiers (side-effect imports like
//! `import 'foo'`). While sometimes needed for polyfills and CSS, they should
//! be used sparingly and intentionally.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags side-effect-only imports that have no specifiers.
#[derive(Debug)]
pub struct RequireModuleSpecifiers;

impl LintRule for RequireModuleSpecifiers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "require-module-specifiers".to_owned(),
            description: "Require import declarations to have specifiers".to_owned(),
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

        // Allow `import type` statements (TypeScript type-only imports)
        if import.import_kind_is_type {
            return;
        }

        // `specifiers` is empty for `import 'foo'` (bare side-effect import)
        let is_side_effect = import.specifiers.is_empty();

        if is_side_effect {
            let source = import.source.as_str();
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove side-effect import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "require-module-specifiers".to_owned(),
                message: format!("Import from '{source}' has no specifiers — side-effect imports should be used sparingly"),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RequireModuleSpecifiers)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_bare_side_effect_import() {
        let diags = lint("import 'foo';");
        assert_eq!(diags.len(), 1, "bare side-effect import should be flagged");
    }

    #[test]
    fn test_flags_polyfill_import() {
        let diags = lint("import './polyfill';");
        assert_eq!(
            diags.len(),
            1,
            "polyfill side-effect import should be flagged"
        );
    }

    #[test]
    fn test_allows_default_import() {
        let diags = lint("import foo from 'foo';");
        assert!(diags.is_empty(), "default import should not be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint("import { foo } from 'foo';");
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_namespace_import() {
        let diags = lint("import * as foo from 'foo';");
        assert!(diags.is_empty(), "namespace import should not be flagged");
    }

    #[test]
    fn test_allows_type_import() {
        let diags = lint("import type { Foo } from 'foo';");
        assert!(diags.is_empty(), "type-only import should not be flagged");
    }
}
