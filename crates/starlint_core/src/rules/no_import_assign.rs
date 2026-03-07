//! Rule: `no-import-assign` (eslint)
//!
//! Disallow reassignment of imported bindings. Import bindings are
//! read-only; attempting to reassign them throws a runtime error.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};
use starlint_scope::SymbolFlags;

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags reassignment of imported bindings.
#[derive(Debug)]
pub struct NoImportAssign;

impl LintRule for NoImportAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-import-assign".to_owned(),
            description: "Disallow reassignment of imported bindings".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        if import.specifiers.is_empty() {
            return;
        }

        let Some(scope_data) = ctx.scope_data() else {
            return;
        };

        for &spec_id in &*import.specifiers {
            // Resolve the specifier node to get the local binding name
            let local_info = match ctx.node(spec_id) {
                Some(AstNode::ImportSpecifier(s)) => Some((s.local.clone(), s.span)),
                Some(AstNode::BindingIdentifier(id)) => Some((id.name.clone(), id.span)),
                _ => None,
            };

            let Some((local_name, local_span)) = local_info else {
                continue;
            };

            // Find the symbol by name in the root scope
            let root_scope = scope_data.root_scope_id();
            let Some(symbol_id) = scope_data.get_binding(root_scope, &local_name) else {
                continue;
            };

            let flags = scope_data.symbol_flags(symbol_id);
            if !flags.contains(SymbolFlags::IMPORT) {
                continue;
            }

            // Check if any reference to this symbol is a write
            let has_write = scope_data
                .get_resolved_references(symbol_id)
                .iter()
                .any(|r| r.flags.is_write());

            if has_write {
                ctx.report(Diagnostic {
                    rule_name: "no-import-assign".to_owned(),
                    message: format!(
                        "'{local_name}' is an imported binding and cannot be reassigned"
                    ),
                    span: Span::new(local_span.start, local_span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoImportAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_import_reassignment() {
        let diags = lint("import foo from 'bar'; foo = 1;");
        assert_eq!(diags.len(), 1, "reassigning import should be flagged");
    }

    #[test]
    fn test_allows_import_read() {
        let diags = lint("import foo from 'bar'; console.log(foo);");
        assert!(diags.is_empty(), "reading import should not be flagged");
    }

    #[test]
    fn test_flags_named_import_reassignment() {
        let diags = lint("import { foo } from 'bar'; foo = 1;");
        assert_eq!(diags.len(), 1, "reassigning named import should be flagged");
    }

    #[test]
    fn test_allows_import_call() {
        let diags = lint("import foo from 'bar'; foo();");
        assert!(diags.is_empty(), "calling import should not be flagged");
    }
}
