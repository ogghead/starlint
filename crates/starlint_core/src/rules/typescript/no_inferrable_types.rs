//! Rule: `typescript/no-inferrable-types`
//!
//! Disallow explicit type annotations on variables where the type can be
//! trivially inferred from the initializer. For example, `let x: number = 5`
//! is redundant because TypeScript already infers `number` from the literal `5`.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Expression, TSType};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags explicit type annotations that match trivially inferred types.
#[derive(Debug)]
pub struct NoInferrableTypes;

impl NativeRule for NoInferrableTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-inferrable-types".to_owned(),
            description:
                "Disallow explicit type annotations on variables with trivially inferred types"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclarator])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclarator(decl) = kind else {
            return;
        };

        // Must have both a type annotation and an initializer
        let Some(type_ann) = &decl.type_annotation else {
            return;
        };
        let Some(init) = &decl.init else {
            return;
        };

        // Must be a simple binding identifier (not destructuring)
        if !matches!(&decl.id, BindingPattern::BindingIdentifier(_)) {
            return;
        }

        if is_inferrable_annotation(&type_ann.type_annotation, init) {
            let type_name = annotation_type_name(&type_ann.type_annotation);
            ctx.report_warning(
                "typescript/no-inferrable-types",
                &format!("Type `{type_name}` is trivially inferred from the initializer"),
                Span::new(decl.span.start, decl.span.end),
            );
        }
    }
}

/// Check if a type annotation is trivially inferrable from the initializer.
///
/// Returns `true` when the annotation is a keyword type (`number`, `string`,
/// `boolean`) and the initializer is the corresponding literal type.
const fn is_inferrable_annotation(ts_type: &TSType<'_>, init: &Expression<'_>) -> bool {
    matches!(
        (ts_type, init),
        (TSType::TSNumberKeyword(_), Expression::NumericLiteral(_))
            | (TSType::TSStringKeyword(_), Expression::StringLiteral(_))
            | (TSType::TSBooleanKeyword(_), Expression::BooleanLiteral(_))
    )
}

/// Get a human-readable name for a type annotation keyword.
const fn annotation_type_name(ts_type: &TSType<'_>) -> &'static str {
    match ts_type {
        TSType::TSNumberKeyword(_) => "number",
        TSType::TSStringKeyword(_) => "string",
        TSType::TSBooleanKeyword(_) => "boolean",
        _ => "unknown",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInferrableTypes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_number_with_numeric_literal() {
        let diags = lint("let x: number = 5;");
        assert_eq!(
            diags.len(),
            1,
            "`let x: number = 5` should be flagged as inferrable"
        );
    }

    #[test]
    fn test_flags_string_with_string_literal() {
        let diags = lint(r#"let x: string = "hello";"#);
        assert_eq!(
            diags.len(),
            1,
            r#"`let x: string = "hello"` should be flagged as inferrable"#
        );
    }

    #[test]
    fn test_flags_boolean_with_boolean_literal() {
        let diags = lint("let x: boolean = true;");
        assert_eq!(
            diags.len(),
            1,
            "`let x: boolean = true` should be flagged as inferrable"
        );
    }

    #[test]
    fn test_allows_type_annotation_without_init() {
        let diags = lint("let x: number;");
        assert!(
            diags.is_empty(),
            "type annotation without initializer should not be flagged"
        );
    }

    #[test]
    fn test_allows_init_without_type_annotation() {
        let diags = lint("let x = 5;");
        assert!(
            diags.is_empty(),
            "initializer without type annotation should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_trivial_type() {
        let diags = lint("let x: Foo = new Foo();");
        assert!(
            diags.is_empty(),
            "non-trivial type annotation should not be flagged"
        );
    }
}
