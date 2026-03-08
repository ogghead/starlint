//! Rule: `import/export`
//!
//! Report any invalid exports, specifically duplicate named exports from
//! the same module. Having two exports with the same name is a syntax error
//! in some environments and always a logical error.

#![allow(clippy::collapsible_if, clippy::collapsible_match)]
use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags duplicate named export declarations within the same module.
#[derive(Debug)]
pub struct ExportRule;

impl LintRule for ExportRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/export".to_owned(),
            description: "Report any invalid exports (duplicate named exports)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Program])
    }

    #[allow(clippy::collapsible_if)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Program(program) = node else {
            return;
        };

        let mut seen_names: HashSet<String> = HashSet::new();

        for &stmt_id in &*program.body {
            let Some(stmt) = ctx.node(stmt_id) else {
                continue;
            };
            // Extract data from the immutable borrow before calling ctx.report()
            let stmt_info = match stmt {
                AstNode::ExportNamedDeclaration(export) => {
                    let specifiers = export.specifiers.clone();
                    let export_span = export.span;
                    let declaration = export.declaration;
                    Some((specifiers, export_span, declaration))
                }
                AstNode::ExportDefaultDeclaration(export) => {
                    if !seen_names.insert("default".to_owned()) {
                        ctx.report(Diagnostic {
                            rule_name: "import/export".to_owned(),
                            message: "Multiple default exports".to_owned(),
                            span: Span::new(export.span.start, export.span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                    None
                }
                _ => None,
            };

            if let Some((specifiers, export_span, declaration)) = stmt_info {
                // Collect specifier data first, then report
                let spec_data: Vec<(String, starlint_ast::types::Span)> = specifiers
                    .iter()
                    .filter_map(|&spec_id| {
                        let AstNode::ExportSpecifier(spec) = ctx.node(spec_id)? else {
                            return None;
                        };
                        Some((spec.exported.clone(), spec.span))
                    })
                    .collect();

                for (exported_name, spec_span) in spec_data {
                    if !seen_names.insert(exported_name.clone()) {
                        ctx.report(Diagnostic {
                            rule_name: "import/export".to_owned(),
                            message: format!("Multiple exports of name '{exported_name}'"),
                            span: Span::new(spec_span.start, spec_span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }

                // Collect names from declaration
                if let Some(decl_id) = declaration {
                    let decl_names = collect_declaration_names(decl_id, ctx);
                    for name in decl_names {
                        if !seen_names.insert(name.clone()) {
                            ctx.report(Diagnostic {
                                rule_name: "import/export".to_owned(),
                                message: format!("Multiple exports of name '{name}'"),
                                span: Span::new(export_span.start, export_span.end),
                                severity: Severity::Error,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Extract binding names from a declaration node.
fn collect_declaration_names(decl_id: NodeId, ctx: &LintContext<'_>) -> Vec<String> {
    let mut names = Vec::new();
    let Some(decl) = ctx.node(decl_id) else {
        return names;
    };
    match decl {
        AstNode::VariableDeclaration(var_decl) => {
            for &declarator_id in &*var_decl.declarations {
                if let Some(AstNode::VariableDeclarator(declarator)) = ctx.node(declarator_id) {
                    if let Some(AstNode::BindingIdentifier(id)) = ctx.node(declarator.id) {
                        names.push(id.name.clone());
                    }
                }
            }
        }
        AstNode::Function(func) => {
            if let Some(id_node) = func.id.and_then(|id| ctx.node(id)) {
                if let AstNode::BindingIdentifier(id) = id_node {
                    names.push(id.name.clone());
                }
            }
        }
        AstNode::Class(class) => {
            if let Some(id_node) = class.id.and_then(|id| ctx.node(id)) {
                if let AstNode::BindingIdentifier(id) = id_node {
                    names.push(id.name.clone());
                }
            }
        }
        AstNode::TSEnumDeclaration(e) => {
            if let Some(AstNode::BindingIdentifier(id)) = ctx.node(e.id) {
                names.push(id.name.clone());
            }
        }
        AstNode::TSInterfaceDeclaration(i) => {
            if let Some(AstNode::BindingIdentifier(id)) = ctx.node(i.id) {
                names.push(id.name.clone());
            }
        }
        AstNode::TSTypeAliasDeclaration(t) => {
            if let Some(AstNode::BindingIdentifier(id)) = ctx.node(t.id) {
                names.push(id.name.clone());
            }
        }
        _ => {}
    }
    names
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExportRule)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_duplicate_named_export() {
        let source = "export const foo = 1;\nexport const foo = 2;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate named export should be flagged");
    }

    #[test]
    fn test_allows_unique_exports() {
        let source = "export const foo = 1;\nexport const bar = 2;";
        let diags = lint(source);
        assert!(diags.is_empty(), "unique exports should not be flagged");
    }

    #[test]
    fn test_flags_duplicate_default_export() {
        let source = "export default 1;\nexport default 2;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate default export should be flagged");
    }

    // --- TypeScript declaration export tests ---

    fn lint_ts(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ExportRule)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_duplicate_exported_enum() {
        let source = "export enum Foo { A }\nexport enum Foo { B }";
        let diags = lint_ts(source);
        assert_eq!(
            diags.len(),
            1,
            "duplicate exported enum declaration should be flagged"
        );
    }

    #[test]
    fn test_flags_duplicate_exported_interface() {
        let source = "export interface Foo { x: number }\nexport interface Foo { y: string }";
        let diags = lint_ts(source);
        assert_eq!(
            diags.len(),
            1,
            "duplicate exported interface declaration should be flagged"
        );
    }

    #[test]
    fn test_flags_duplicate_exported_type_alias() {
        let source = "export type Foo = string;\nexport type Foo = number;";
        let diags = lint_ts(source);
        assert_eq!(
            diags.len(),
            1,
            "duplicate exported type alias declaration should be flagged"
        );
    }
}
