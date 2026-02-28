//! Rule: `typescript/no-explicit-any`
//!
//! Disallow the `any` type annotation. Using `any` disables `TypeScript` type
//! checking for the annotated binding, defeating the purpose of the type system.
//! Prefer `unknown`, generics, or explicit types instead.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of the `any` type annotation.
#[derive(Debug)]
pub struct NoExplicitAny;

impl NativeRule for NoExplicitAny {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-explicit-any".to_owned(),
            description: "Disallow the `any` type annotation".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSAnyKeyword(keyword) = kind else {
            return;
        };

        ctx.report_warning(
            "typescript/no-explicit-any",
            "Unexpected `any` type annotation — use `unknown` or a specific type instead",
            Span::new(keyword.span.start, keyword.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExplicitAny)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_any_variable() {
        let diags = lint("let x: any;");
        assert_eq!(diags.len(), 1, "`any` type annotation should be flagged");
    }

    #[test]
    fn test_flags_any_parameter() {
        let diags = lint("function f(x: any) {}");
        assert_eq!(
            diags.len(),
            1,
            "`any` in function parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_unknown() {
        let diags = lint("let x: unknown;");
        assert!(diags.is_empty(), "`unknown` should not be flagged");
    }

    #[test]
    fn test_allows_string() {
        let diags = lint("let x: string;");
        assert!(diags.is_empty(), "`string` should not be flagged");
    }
}
