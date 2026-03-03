//! Rule: `typescript/no-unsafe-argument`
//!
//! Disallow calling a function with an `any`-typed argument. Passing `as any`
//! to a function defeats type checking for that parameter position and can
//! hide type errors.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This AST-based rule only detects `as any` expressions passed directly as
//! function call arguments (e.g. `foo(x as any)`).

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, TSType};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unsafe-argument";

/// Flags function call arguments that use `as any` type assertions.
#[derive(Debug)]
pub struct NoUnsafeArgument;

impl NativeRule for NoUnsafeArgument {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow calling a function with an `as any` argument".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        for arg in &call.arguments {
            if is_as_any_argument(arg) {
                let arg_span = argument_span(arg);
                ctx.report_warning(
                    RULE_NAME,
                    "Unsafe `as any` argument — this bypasses type checking for \
                     the corresponding parameter",
                    arg_span,
                );
            }
        }
    }
}

/// Check if a function argument is a `TSAsExpression` casting to `any`.
fn is_as_any_argument(arg: &Argument<'_>) -> bool {
    let Argument::TSAsExpression(ts_as) = arg else {
        return false;
    };
    matches!(ts_as.type_annotation, TSType::TSAnyKeyword(_))
}

/// Extract the span from an `Argument`.
fn argument_span(arg: &Argument<'_>) -> Span {
    use oxc_span::GetSpan;
    let span = arg.span();
    Span::new(span.start, span.end)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeArgument)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_as_any_argument() {
        let diags = lint("declare function foo(x: number): void;\nfoo(value as any);");
        assert_eq!(diags.len(), 1, "`as any` argument should be flagged");
    }

    #[test]
    fn test_flags_multiple_as_any_arguments() {
        let diags =
            lint("declare function bar(a: string, b: number): void;\nbar(x as any, y as any);");
        assert_eq!(diags.len(), 2, "both `as any` arguments should be flagged");
    }

    #[test]
    fn test_allows_as_string_argument() {
        let diags = lint("declare function foo(x: string): void;\nfoo(value as string);");
        assert!(
            diags.is_empty(),
            "`as string` argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_argument() {
        let diags = lint("declare function foo(x: number): void;\nfoo(42);");
        assert!(diags.is_empty(), "normal argument should not be flagged");
    }

    #[test]
    fn test_allows_as_unknown_argument() {
        let diags = lint("declare function foo(x: unknown): void;\nfoo(value as unknown);");
        assert!(
            diags.is_empty(),
            "`as unknown` argument should not be flagged by this rule"
        );
    }
}
