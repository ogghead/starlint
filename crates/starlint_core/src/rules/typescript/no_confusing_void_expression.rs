//! Rule: `typescript/no-confusing-void-expression`
//!
//! Disallow `void` expressions in misleading positions. The `void` operator
//! evaluates to `undefined`, but using `void expr` as a value (in variable
//! initializers, return statements, assignment right-hand sides, or arrow
//! function expression bodies) is confusing and likely a mistake. Use
//! `undefined` explicitly or separate the side-effect call from the value.

use oxc_ast::AstKind;
use oxc_ast::ast::UnaryOperator;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `void` expressions used in value positions.
#[derive(Debug)]
pub struct NoConfusingVoidExpression;

impl NativeRule for NoConfusingVoidExpression {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-confusing-void-expression".to_owned(),
            description: "Disallow `void` expressions in misleading positions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::UnaryExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::UnaryExpression(unary) = kind else {
            return;
        };

        if unary.operator != UnaryOperator::Void {
            return;
        }

        // Check source text before the void expression to detect value positions.
        // This is a heuristic since single-pass traversal lacks parent context.
        let (in_value, arg_text) = {
            let source = ctx.source_text();
            let arg_span = unary.argument.span();
            #[allow(clippy::as_conversions)]
            let arg_start = arg_span.start as usize;
            #[allow(clippy::as_conversions)]
            let arg_end = arg_span.end as usize;
            let text = source.get(arg_start..arg_end).unwrap_or("").to_owned();
            (is_in_value_position(source, unary.span.start), text)
        };

        if in_value {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-confusing-void-expression".to_owned(),
                message: "Void expression used in a value position — use `undefined` or separate the side effect".to_owned(),
                span: Span::new(unary.span.start, unary.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove `void` operator".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(unary.span.start, unary.span.end),
                        replacement: arg_text,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Heuristic check whether a `void` expression at `start` is used as a value.
///
/// Scans backward from the `void` keyword to find tokens that indicate
/// the expression result is being consumed: `=`, `return`, `=>`, `(`, `?`, `:`.
fn is_in_value_position(source: &str, start: u32) -> bool {
    let pos = usize::try_from(start).unwrap_or(0);
    let before = source.get(..pos).unwrap_or("").trim_end();

    // Look at the last significant token before the void expression
    if before.ends_with('=') {
        // Assignment (`x = void 0`) or initializer (`const x = void 0`),
        // but not `==` or `!=` (comparison).
        let prefix = before.get(..before.len().saturating_sub(1)).unwrap_or("");
        let prev_char = prefix.chars().next_back().unwrap_or(' ');
        // `==`, `!=`, `<=`, `>=` are comparisons, not value positions
        return !matches!(prev_char, '=' | '!' | '<' | '>');
    }

    if before.ends_with("return") {
        return true;
    }

    if before.ends_with("=>") {
        return true;
    }

    // Ternary branches: `cond ? void 0 : ...` or `cond ? x : void 0`
    if before.ends_with('?') || before.ends_with(':') {
        return true;
    }

    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConfusingVoidExpression)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_void_in_variable_initializer() {
        let diags = lint("const x = void 0;");
        assert_eq!(
            diags.len(),
            1,
            "void in variable initializer should be flagged"
        );
    }

    #[test]
    fn test_flags_void_in_return_statement() {
        let diags = lint("function f() { return void doSomething(); }");
        assert_eq!(diags.len(), 1, "void in return statement should be flagged");
    }

    #[test]
    fn test_flags_void_in_arrow_body() {
        let diags = lint("const f = () => void doSomething();");
        assert_eq!(
            diags.len(),
            1,
            "void in arrow function body should be flagged"
        );
    }

    #[test]
    fn test_allows_void_as_statement() {
        let diags = lint("void doSomething();");
        assert!(
            diags.is_empty(),
            "void as standalone statement should not be flagged"
        );
    }

    #[test]
    fn test_allows_void_not_in_value_position() {
        let diags = lint("if (true) { void doSomething(); }");
        assert!(
            diags.is_empty(),
            "void not in value position should not be flagged"
        );
    }
}
