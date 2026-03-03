//! Rule: `typescript/no-unsafe-type-assertion`
//!
//! Disallow type assertions that cast to `any` or `unknown`. Using `x as any`
//! or `x as unknown` are escape hatches that bypass TypeScript's type system.
//! These assertions hide potential type errors and make refactoring harder.
//! Prefer explicit type narrowing, generics, or proper type guards instead.

use oxc_ast::AstKind;
use oxc_ast::ast::TSType;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `as any` and `as unknown` type assertions.
#[derive(Debug)]
pub struct NoUnsafeTypeAssertion;

impl NativeRule for NoUnsafeTypeAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-type-assertion".to_owned(),
            description: "Disallow type assertions to `any` or `unknown`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSAsExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSAsExpression(expr) = kind else {
            return;
        };

        let escape_type = match &expr.type_annotation {
            TSType::TSAnyKeyword(_) => "any",
            TSType::TSUnknownKeyword(_) => "unknown",
            _ => return,
        };

        ctx.report_warning(
            "typescript/no-unsafe-type-assertion",
            &format!(
                "Avoid `as {escape_type}` — it bypasses type checking. Use a type guard or explicit type instead"
            ),
            Span::new(expr.span.start, expr.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeTypeAssertion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_as_any() {
        let diags = lint("let x = value as any;");
        assert_eq!(diags.len(), 1, "`as any` assertion should be flagged");
    }

    #[test]
    fn test_flags_as_unknown() {
        let diags = lint("let x = value as unknown;");
        assert_eq!(diags.len(), 1, "`as unknown` assertion should be flagged");
    }

    #[test]
    fn test_allows_as_string() {
        let diags = lint("let x = value as string;");
        assert!(
            diags.is_empty(),
            "`as string` assertion should not be flagged"
        );
    }

    #[test]
    fn test_allows_as_number() {
        let diags = lint("let x = value as number;");
        assert!(
            diags.is_empty(),
            "`as number` assertion should not be flagged"
        );
    }

    #[test]
    fn test_flags_nested_as_any() {
        let diags = lint("let x = (foo.bar() as any).baz;");
        assert_eq!(
            diags.len(),
            1,
            "nested `as any` assertion should be flagged"
        );
    }
}
