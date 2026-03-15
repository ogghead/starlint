//! Rule: `react/only-export-components`
//!
//! Warn when a file exports non-component values alongside components,
//! which breaks Fast Refresh.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags named exports of non-component identifiers (lowercase names).
///
/// Detects files that likely contain React components with non-component
/// named exports, which would break Fast Refresh.
#[derive(Debug)]
pub struct OnlyExportComponents;

/// Check if a name starts with a lowercase letter (not a component by convention).
fn is_non_component_name(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|&b| b.is_ascii_lowercase())
}

impl LintRule for OnlyExportComponents {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/only-export-components".to_owned(),
            description: "Warn when non-component values are exported alongside components"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExportNamedDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ExportNamedDeclaration(export) = node else {
            return;
        };

        // Check specifiers like `export { foo, Bar }`
        // ExportSpecifierNode has `exported: String`
        let spec_violations: Vec<(String, starlint_ast::types::Span)> = export
            .specifiers
            .iter()
            .filter_map(|spec_id| {
                if let Some(AstNode::ExportSpecifier(spec)) = ctx.node(*spec_id) {
                    let name = spec.exported.as_str();
                    if is_non_component_name(name) {
                        return Some((name.to_owned(), spec.span));
                    }
                }
                None
            })
            .collect();

        for (name, span) in spec_violations {
            ctx.report(Diagnostic {
                rule_name: "react/only-export-components".to_owned(),
                message: format!(
                    "Fast Refresh only works when a file exports components. Use a separate file for `{name}`"
                ),
                span: Span::new(span.start, span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }

        // Check inline declarations like `export const foo = ...`
        if let Some(decl_id) = export.declaration {
            match ctx.node(decl_id) {
                Some(AstNode::VariableDeclaration(var_decl)) => {
                    let decl_violations: Vec<(String, starlint_ast::types::Span)> = var_decl
                        .declarations
                        .iter()
                        .filter_map(|declarator_id| {
                            if let Some(AstNode::VariableDeclarator(declarator)) =
                                ctx.node(*declarator_id)
                            {
                                if let Some(AstNode::BindingIdentifier(id)) =
                                    ctx.node(declarator.id)
                                {
                                    let name = id.name.as_str();
                                    if is_non_component_name(name) {
                                        return Some((name.to_owned(), id.span));
                                    }
                                }
                            }
                            None
                        })
                        .collect();

                    for (name, span) in decl_violations {
                        ctx.report(Diagnostic {
                            rule_name: "react/only-export-components".to_owned(),
                            message: format!(
                                "Fast Refresh only works when a file exports components. Use a separate file for `{name}`"
                            ),
                            span: Span::new(span.start, span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
                Some(AstNode::Function(func)) => {
                    if let Some(id_node_id) = func.id {
                        if let Some(AstNode::BindingIdentifier(id)) = ctx.node(id_node_id) {
                            let name = id.name.as_str();
                            if is_non_component_name(name) {
                                ctx.report(Diagnostic {
                                    rule_name: "react/only-export-components".to_owned(),
                                    message: format!(
                                        "Fast Refresh only works when a file exports components. Use a separate file for `{name}`"
                                    ),
                                    span: Span::new(id.span.start, id.span.end),
                                    severity: Severity::Warning,
                                    help: None,
                                    fix: None,
                                    labels: vec![],
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(OnlyExportComponents);

    #[test]
    fn test_flags_lowercase_named_export() {
        let source = "export const myHelper = () => 42;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "lowercase named export should be flagged");
    }

    #[test]
    fn test_allows_uppercase_named_export() {
        let source = "export const MyComponent = () => <div />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "uppercase named export should not be flagged"
        );
    }

    #[test]
    fn test_flags_lowercase_specifier_export() {
        let source = "const foo = 1;\nconst Bar = () => <div />;\nexport { foo, Bar };";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "lowercase specifier export should be flagged"
        );
    }
}
