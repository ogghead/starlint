//! Rule: `import/export`
//!
//! Report any invalid exports, specifically duplicate named exports from
//! the same module. Having two exports with the same name is a syntax error
//! in some environments and always a logical error.

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags duplicate named export declarations within the same module.
#[derive(Debug)]
pub struct ExportRule;

impl NativeRule for ExportRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/export".to_owned(),
            description: "Report any invalid exports (duplicate named exports)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Program])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Program(program) = kind else {
            return;
        };

        let mut seen_names: HashSet<String> = HashSet::new();

        for stmt in &program.body {
            match stmt {
                oxc_ast::ast::Statement::ExportNamedDeclaration(export) => {
                    // Collect names from specifiers
                    for spec in &export.specifiers {
                        let exported_name = spec.exported.name().as_str();
                        if !seen_names.insert(exported_name.to_owned()) {
                            ctx.report(Diagnostic {
                                rule_name: "import/export".to_owned(),
                                message: format!("Multiple exports of name '{exported_name}'"),
                                span: Span::new(spec.span.start, spec.span.end),
                                severity: Severity::Error,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }

                    // Collect names from declaration
                    if let Some(decl) = &export.declaration {
                        for name in collect_declaration_names(decl) {
                            if !seen_names.insert(name.clone()) {
                                ctx.report(Diagnostic {
                                    rule_name: "import/export".to_owned(),
                                    message: format!("Multiple exports of name '{name}'"),
                                    span: Span::new(export.span.start, export.span.end),
                                    severity: Severity::Error,
                                    help: None,
                                    fix: None,
                                    labels: vec![],
                                });
                            }
                        }
                    }
                }
                oxc_ast::ast::Statement::ExportDefaultDeclaration(export) => {
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
                }
                _ => {}
            }
        }
    }
}

/// Extract binding names from a declaration.
fn collect_declaration_names(decl: &oxc_ast::ast::Declaration<'_>) -> Vec<String> {
    let mut names = Vec::new();
    match decl {
        oxc_ast::ast::Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id {
                    names.push(id.name.as_str().to_owned());
                }
            }
        }
        oxc_ast::ast::Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                names.push(id.name.as_str().to_owned());
            }
        }
        oxc_ast::ast::Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                names.push(id.name.as_str().to_owned());
            }
        }
        oxc_ast::ast::Declaration::TSEnumDeclaration(e) => {
            names.push(e.id.name.as_str().to_owned());
        }
        oxc_ast::ast::Declaration::TSInterfaceDeclaration(i) => {
            names.push(i.id.name.as_str().to_owned());
        }
        oxc_ast::ast::Declaration::TSTypeAliasDeclaration(t) => {
            names.push(t.id.name.as_str().to_owned());
        }
        _ => {}
    }
    names
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExportRule)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
}
