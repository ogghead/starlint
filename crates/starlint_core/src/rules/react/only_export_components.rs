//! Rule: `react/only-export-components`
//!
//! Warn when a file exports non-component values alongside components,
//! which breaks Fast Refresh.

use oxc_ast::AstKind;
use oxc_ast::ast::{Declaration, ExportSpecifier, ModuleExportName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

/// Extract the exported name from a module export name node.
fn get_export_name<'a>(name: &'a ModuleExportName<'a>) -> &'a str {
    match name {
        ModuleExportName::IdentifierName(id) => id.name.as_str(),
        ModuleExportName::IdentifierReference(id) => id.name.as_str(),
        ModuleExportName::StringLiteral(s) => s.value.as_str(),
    }
}

impl NativeRule for OnlyExportComponents {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/only-export-components".to_owned(),
            description: "Warn when non-component values are exported alongside components"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExportNamedDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ExportNamedDeclaration(export) = kind else {
            return;
        };

        // Check specifiers like `export { foo, Bar }`
        for spec in &export.specifiers {
            check_specifier(spec, ctx);
        }

        // Check inline declarations like `export const foo = ...`
        if let Some(decl) = &export.declaration {
            match decl {
                Declaration::VariableDeclaration(var_decl) => {
                    for declarator in &var_decl.declarations {
                        if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id
                        {
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
                Declaration::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id {
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
                _ => {}
            }
        }
    }
}

/// Check a single export specifier and report if it exports a non-component name.
fn check_specifier(spec: &ExportSpecifier<'_>, ctx: &mut NativeLintContext<'_>) {
    let name = get_export_name(&spec.exported);
    if is_non_component_name(name) {
        ctx.report(Diagnostic {
            rule_name: "react/only-export-components".to_owned(),
            message: format!(
                "Fast Refresh only works when a file exports components. Use a separate file for `{name}`"
            ),
            span: Span::new(spec.span.start, spec.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(OnlyExportComponents)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

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
