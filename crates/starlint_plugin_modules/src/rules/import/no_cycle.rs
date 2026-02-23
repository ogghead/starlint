//! Rule: `import/no-cycle`
//!
//! Detect circular import dependencies. Full cycle detection requires
//! module resolution across the entire dependency graph, which is not yet
//! available. As a useful stub, this rule flags self-imports — a module
//! importing from its own file path — which is the simplest form of cycle.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags self-imports as the simplest detectable import cycle.
#[derive(Debug)]
pub struct NoCycle;

impl LintRule for NoCycle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-cycle".to_owned(),
            description: "Detect circular import dependencies (stub: flags self-imports)"
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

        let source_value = import.source.as_str();

        // Only check relative imports
        if !source_value.starts_with('.') {
            return;
        }

        // Extract the file stem of the current file
        let file_path = ctx.file_path();
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        // Extract the last segment of the import path (after the final /)
        let import_segment = source_value.rsplit('/').next().unwrap_or(source_value);

        // Strip extension from import segment if present
        let import_base = import_segment.rfind('.').map_or(import_segment, |pos| {
            import_segment.get(..pos).unwrap_or(import_segment)
        });

        // Self-import: the import source resolves to the same file
        // Match patterns like `./myModule`, `./myModule.ts`, `./myModule.js`
        let is_self_import = !file_stem.is_empty()
            && import_base == file_stem
            && (source_value == format!("./{file_stem}")
                || source_value == format!("./{file_stem}.ts")
                || source_value == format!("./{file_stem}.js")
                || source_value == format!("./{file_stem}.tsx")
                || source_value == format!("./{file_stem}.jsx"));

        if is_self_import {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove self-import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-cycle".to_owned(),
                message: "Module imports itself, creating a circular dependency".to_owned(),
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

    fn lint_with_path(
        source: &str,
        path: &str,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCycle)];
        lint_source(source, path, &rules)
    }

    #[test]
    fn test_flags_self_import() {
        let diags = lint_with_path(r#"import { foo } from "./myModule";"#, "myModule.ts");
        assert_eq!(diags.len(), 1, "self-import should be flagged as a cycle");
    }

    #[test]
    fn test_allows_different_module() {
        let diags = lint_with_path(r#"import { foo } from "./other";"#, "myModule.ts");
        assert!(
            diags.is_empty(),
            "importing a different module should not be flagged"
        );
    }

    #[test]
    fn test_allows_bare_specifier() {
        let diags = lint_with_path(r#"import { foo } from "lodash";"#, "myModule.ts");
        assert!(diags.is_empty(), "bare specifier should not be flagged");
    }
}
