//! Rule: `typescript/explicit-module-boundary-types`
//!
//! Require explicit types on exported functions and class methods. Public API
//! boundaries should have explicit types for documentation and stability.
//! Without explicit types, internal refactoring can accidentally change the
//! public contract.

use oxc_ast::AstKind;
use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/explicit-module-boundary-types";

/// Flags exported functions and class methods that lack explicit return type
/// or parameter type annotations.
#[derive(Debug)]
pub struct ExplicitModuleBoundaryTypes;

impl NativeRule for ExplicitModuleBoundaryTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require explicit types on exported functions and class methods"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ExportDefaultDeclaration,
            AstType::ExportNamedDeclaration,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::ExportNamedDeclaration(decl) => {
                if let Some(declaration) = &decl.declaration {
                    check_declaration(declaration, ctx);
                }
            }
            AstKind::ExportDefaultDeclaration(decl) => {
                check_default_declaration(&decl.declaration, decl.span.start, decl.span.end, ctx);
            }
            _ => {}
        }
    }
}

/// Check a named export declaration for missing type annotations.
fn check_declaration(decl: &Declaration<'_>, ctx: &mut NativeLintContext<'_>) {
    if let Declaration::FunctionDeclaration(func) = decl {
        // Skip functions without a body (ambient declarations)
        if func.body.is_none() {
            return;
        }

        if func.return_type.is_none() {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Exported function missing explicit return type".to_owned(),
                span: Span::new(func.span.start, func.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }

        // Check each parameter for a type annotation
        for param in &func.params.items {
            if param.type_annotation.is_none() {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Exported function parameter missing explicit type annotation"
                        .to_owned(),
                    span: Span::new(param.span.start, param.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
    }
}

/// Check a default export declaration for missing type annotations.
fn check_default_declaration(
    decl: &ExportDefaultDeclarationKind<'_>,
    span_start: u32,
    span_end: u32,
    ctx: &mut NativeLintContext<'_>,
) {
    match decl {
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            // Skip functions without a body (ambient declarations)
            if func.body.is_none() {
                return;
            }

            if func.return_type.is_none() {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Exported function missing explicit return type".to_owned(),
                    span: Span::new(func.span.start, func.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }

            for param in &func.params.items {
                if param.type_annotation.is_none() {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Exported function parameter missing explicit type annotation"
                            .to_owned(),
                        span: Span::new(param.span.start, param.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            if arrow.return_type.is_none() {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Exported arrow function missing explicit return type".to_owned(),
                    span: Span::new(span_start, span_end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }

            for param in &arrow.params.items {
                if param.type_annotation.is_none() {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Exported function parameter missing explicit type annotation"
                            .to_owned(),
                        span: Span::new(param.span.start, param.span.end),
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExplicitModuleBoundaryTypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_exported_function_missing_return_type() {
        let diags = lint("export function foo() { return 1; }");
        assert!(
            !diags.is_empty(),
            "exported function without return type should be flagged"
        );
    }

    #[test]
    fn test_allows_exported_function_with_return_type() {
        let diags = lint("export function foo(): number { return 1; }");
        assert!(
            diags.is_empty(),
            "exported function with return type should not be flagged"
        );
    }

    #[test]
    fn test_flags_exported_function_missing_param_type() {
        let diags = lint("export function foo(x): number { return x; }");
        assert_eq!(
            diags.len(),
            1,
            "exported function with untyped parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_non_exported_function() {
        let diags = lint("function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "non-exported function should not be flagged"
        );
    }

    #[test]
    fn test_flags_default_exported_function_missing_return_type() {
        let diags = lint("export default function foo() { return 1; }");
        assert!(
            !diags.is_empty(),
            "default-exported function without return type should be flagged"
        );
    }
}
