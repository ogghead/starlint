//! Rule: `no-duplicate-imports`
//!
//! Disallow duplicate module imports. If a module is imported more than
//! once, the imports should be merged into a single import statement.

use std::collections::HashMap;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags duplicate import declarations from the same module.
#[derive(Debug)]
pub struct NoDuplicateImports;

impl LintRule for NoDuplicateImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-duplicate-imports".to_owned(),
            description: "Disallow duplicate module imports".to_owned(),
            category: Category::Suggestion,
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

        // Collect import data upfront to avoid borrow conflicts
        let imports: Vec<(String, Span)> = program
            .body
            .iter()
            .filter_map(|&stmt_id| {
                let import = ctx.node(stmt_id)?.as_import_declaration()?;
                Some((
                    import.source.clone(),
                    Span::new(import.span.start, import.span.end),
                ))
            })
            .collect();

        // Map module source → first import span
        let mut seen: HashMap<String, Span> = HashMap::new();
        let mut duplicates: Vec<(String, Span, Span)> = Vec::new();

        for (source_str, import_span) in &imports {
            if let Some(&first_span) = seen.get(source_str.as_str()) {
                duplicates.push((source_str.clone(), first_span, *import_span));
            } else {
                seen.insert(source_str.clone(), *import_span);
            }
        }

        // Build fixes first (immutable borrow of source_text), then report
        let diagnostics: Vec<Diagnostic> = {
            let source_text = ctx.source_text();
            duplicates
                .iter()
                .map(|(module_source, first_span, dup_span)| {
                    let edits = fix_utils::merge_import_edits(source_text, *first_span, *dup_span);
                    let fix = FixBuilder::new("Merge into first import", FixKind::SuggestionFix)
                        .edits(edits)
                        .build();
                    Diagnostic {
                        rule_name: "no-duplicate-imports".to_owned(),
                        message: format!("'{module_source}' import is duplicated"),
                        span: *dup_span,
                        severity: Severity::Warning,
                        help: None,
                        fix,
                        labels: vec![],
                    }
                })
                .collect()
        };
        for diag in diagnostics {
            ctx.report(diag);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDuplicateImports)];
        lint_source(source, "test.mjs", &rules)
    }

    #[test]
    fn test_flags_duplicate_import() {
        let diags = lint("import { a } from 'foo';\nimport { b } from 'foo';");
        assert_eq!(diags.len(), 1, "duplicate import should be flagged");
    }

    #[test]
    fn test_allows_different_sources() {
        let diags = lint("import { a } from 'foo';\nimport { b } from 'bar';");
        assert!(diags.is_empty(), "different sources should not be flagged");
    }

    #[test]
    fn test_allows_single_import() {
        let diags = lint("import { a, b } from 'foo';");
        assert!(diags.is_empty(), "single import should not be flagged");
    }

    #[test]
    fn test_flags_triple_import() {
        let diags =
            lint("import { a } from 'foo';\nimport { b } from 'foo';\nimport { c } from 'foo';");
        assert_eq!(
            diags.len(),
            2,
            "two duplicate imports should produce two diagnostics"
        );
    }
}
