//! Rule: `typescript/consistent-type-assertions`
//!
//! Prefer `as` syntax over angle-bracket syntax for type assertions. The
//! angle-bracket form (`<Type>expr`) is ambiguous with JSX in `.tsx` files
//! and is less common in modern TypeScript codebases. Using `as` consistently
//! avoids confusion and ensures compatibility with JSX.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags angle-bracket type assertions (`<Type>expr`), preferring `as` syntax.
#[derive(Debug)]
pub struct ConsistentTypeAssertions;

impl NativeRule for ConsistentTypeAssertions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-type-assertions".to_owned(),
            description: "Prefer `as` syntax for type assertions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSTypeAssertion])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeAssertion(expr) = kind else {
            return;
        };

        // Build fix: rewrite <Type>expression to expression as Type
        let source = ctx.source_text();
        let type_span = expr.type_annotation.span();
        let expr_span = expr.expression.span();
        let type_text = source
            .get(type_span.start as usize..type_span.end as usize)
            .unwrap_or("")
            .to_owned();
        let expr_text = source
            .get(expr_span.start as usize..expr_span.end as usize)
            .unwrap_or("")
            .to_owned();

        let fix = (!type_text.is_empty() && !expr_text.is_empty()).then(|| Fix {
            message: format!("Rewrite to `{expr_text} as {type_text}`"),
            edits: vec![Edit {
                span: Span::new(expr.span.start, expr.span.end),
                replacement: format!("{expr_text} as {type_text}"),
            }],
        });

        ctx.report(Diagnostic {
            rule_name: "typescript/consistent-type-assertions".to_owned(),
            message: "Use `as` syntax instead of angle-bracket syntax for type assertions"
                .to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Use `as` syntax instead".to_owned()),
            fix,
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentTypeAssertions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_angle_bracket_assertion() {
        let diags = lint("let x = <string>someValue;");
        assert_eq!(
            diags.len(),
            1,
            "angle-bracket type assertion should be flagged"
        );
    }

    #[test]
    fn test_flags_angle_bracket_with_complex_type() {
        let diags = lint("let x = <Array<number>>someValue;");
        assert_eq!(
            diags.len(),
            1,
            "angle-bracket assertion with generic type should be flagged"
        );
    }

    #[test]
    fn test_allows_as_syntax() {
        let diags = lint("let x = someValue as string;");
        assert!(
            diags.is_empty(),
            "`as` syntax assertion should not be flagged"
        );
    }

    #[test]
    fn test_allows_as_const() {
        let diags = lint(r#"let x = "hello" as const;"#);
        assert!(diags.is_empty(), "`as const` should not be flagged");
    }

    #[test]
    fn test_allows_no_assertion() {
        let diags = lint("let x: string = someValue;");
        assert!(
            diags.is_empty(),
            "type annotation without assertion should not be flagged"
        );
    }
}
