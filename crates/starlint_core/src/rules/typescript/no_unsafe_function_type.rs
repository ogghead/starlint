//! Rule: `typescript/no-unsafe-function-type`
//!
//! Disallow the `Function` type. The `Function` type accepts any function-like
//! value and provides no type safety for calling the value — arguments and
//! return type are all `any`. Prefer specific function signatures like
//! `() => void`, `(arg: string) => number`, or the `(...args: any[]) => any`
//! escape hatch when the signature is truly unknown.

use oxc_ast::AstKind;
use oxc_ast::ast::TSTypeName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of the `Function` type in type annotations.
#[derive(Debug)]
pub struct NoUnsafeFunctionType;

impl NativeRule for NoUnsafeFunctionType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-function-type".to_owned(),
            description: "Disallow the `Function` type — use a specific function signature instead"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSTypeReference])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeReference(type_ref) = kind else {
            return;
        };

        let TSTypeName::IdentifierReference(ident) = &type_ref.type_name else {
            return;
        };

        if ident.name.as_str() != "Function" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/no-unsafe-function-type".to_owned(),
            message: "The `Function` type is unsafe — use a specific function type like `() => void` instead".to_owned(),
            span: Span::new(type_ref.span.start, type_ref.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `(...args: any[]) => any`".to_owned()),
            fix: Some(Fix {
                message: "Replace with `(...args: any[]) => any`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(type_ref.span.start, type_ref.span.end),
                    replacement: "(...args: any[]) => any".to_owned(),
                }],
                is_snippet: false,
            }),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeFunctionType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_function_variable() {
        let diags = lint("let f: Function;");
        assert_eq!(diags.len(), 1, "`Function` type should be flagged");
    }

    #[test]
    fn test_flags_function_parameter() {
        let diags = lint("function run(cb: Function) {}");
        assert_eq!(
            diags.len(),
            1,
            "`Function` as parameter type should be flagged"
        );
    }

    #[test]
    fn test_flags_function_return_type() {
        let diags = lint("function factory(): Function { return () => {}; }");
        assert_eq!(
            diags.len(),
            1,
            "`Function` as return type should be flagged"
        );
    }

    #[test]
    fn test_allows_specific_function_type() {
        let diags = lint("let f: () => void;");
        assert!(
            diags.is_empty(),
            "specific function type should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_with_args() {
        let diags = lint("let f: (x: number) => string;");
        assert!(
            diags.is_empty(),
            "typed function signature should not be flagged"
        );
    }
}
