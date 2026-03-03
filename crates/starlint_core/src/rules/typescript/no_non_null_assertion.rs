//! Rule: `typescript/no-non-null-assertion`
//!
//! Disallow non-null assertions using the `!` postfix operator. The non-null
//! assertion operator (`x!`) tells `TypeScript` to treat a value as non-null
//! without any runtime check, which can mask potential `null`/`undefined` bugs.
//! Prefer optional chaining (`?.`) or explicit null checks instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags non-null assertion expressions (`!` postfix operator).
#[derive(Debug)]
pub struct NoNonNullAssertion;

impl NativeRule for NoNonNullAssertion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-non-null-assertion".to_owned(),
            description: "Disallow non-null assertions using the `!` postfix operator".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSNonNullExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSNonNullExpression(expr) = kind else {
            return;
        };

        ctx.report_warning(
            "typescript/no-non-null-assertion",
            "Avoid non-null assertions — use optional chaining or explicit null checks instead",
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNonNullAssertion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_non_null_member_access() {
        let diags = lint("declare const x: { foo: string } | null; x!.foo;");
        assert_eq!(
            diags.len(),
            1,
            "`x!.foo` non-null assertion should be flagged"
        );
    }

    #[test]
    fn test_flags_non_null_standalone() {
        let diags = lint("declare const x: string | null; x!;");
        assert_eq!(
            diags.len(),
            1,
            "standalone `x!` non-null assertion should be flagged"
        );
    }

    #[test]
    fn test_allows_optional_chaining() {
        let diags = lint("declare const x: { foo: string } | null; x?.foo;");
        assert!(diags.is_empty(), "optional chaining should not be flagged");
    }

    #[test]
    fn test_allows_normal_member_access() {
        let diags = lint("declare const x: { foo: string }; x.foo;");
        assert!(
            diags.is_empty(),
            "normal member access should not be flagged"
        );
    }
}
