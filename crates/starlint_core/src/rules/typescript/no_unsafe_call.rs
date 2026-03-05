//! Rule: `typescript/no-unsafe-call`
//!
//! Disallow calling `any` typed values. Calling a value cast to `any`
//! bypasses all parameter and return type checking, allowing runtime
//! type errors to go undetected at compile time.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This rule detects explicit `(expr as any)(...)` patterns.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, TSType};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags call expressions where the callee is cast to `any`.
#[derive(Debug)]
pub struct NoUnsafeCall;

impl NativeRule for NoUnsafeCall {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-call".to_owned(),
            description: "Disallow calling `any` typed values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        if is_as_any_callee(&call.callee) {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-call".to_owned(),
                message: "Unsafe call — calling an `as any` expression bypasses argument and return type checking".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check whether a callee expression is an `as any` cast, unwrapping
/// parenthesized expressions.
fn is_as_any_callee(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::TSAsExpression(as_expr) => {
            matches!(&as_expr.type_annotation, TSType::TSAnyKeyword(_))
        }
        Expression::ParenthesizedExpression(paren) => is_as_any_callee(&paren.expression),
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeCall)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_call_as_any() {
        let diags = lint("(getValue as any)();");
        assert_eq!(diags.len(), 1, "`(getValue as any)()` should be flagged");
    }

    #[test]
    fn test_flags_nested_paren_call_as_any() {
        let diags = lint("((fn as any))();");
        assert_eq!(
            diags.len(),
            1,
            "double-parenthesized `as any` call should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_call() {
        let diags = lint("getValue();");
        assert!(diags.is_empty(), "normal call should not be flagged");
    }

    #[test]
    fn test_allows_call_as_string() {
        let diags = lint("(getValue as Function)();");
        assert!(
            diags.is_empty(),
            "`as Function` call should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_typed_call() {
        let diags = lint("(getValue as () => number)();");
        assert!(
            diags.is_empty(),
            "typed function call should not be flagged"
        );
    }
}
