//! Rule: `no-useless-fallback-in-spread` (unicorn)
//!
//! Disallow useless fallback when spreading in object literals.
//! `{...(obj || {})}` and `{...(obj ?? {})}` are unnecessary because
//! spreading `undefined`/`null` in object literals is a no-op.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `{...(obj || {})}` and `{...(obj ?? {})}` patterns.
#[derive(Debug)]
pub struct NoUselessFallbackInSpread;

impl NativeRule for NoUselessFallbackInSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-fallback-in-spread".to_owned(),
            description: "Disallow useless fallback in spread".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SpreadElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SpreadElement(spread) = kind else {
            return;
        };

        // Check for `(obj || {})` or `(obj ?? {})` in spread
        let expr = unwrap_parens(&spread.argument);

        let Expression::LogicalExpression(logical) = expr else {
            return;
        };

        // Must be `||` or `??`
        if !matches!(
            logical.operator,
            oxc_ast::ast::LogicalOperator::Or | oxc_ast::ast::LogicalOperator::Coalesce
        ) {
            return;
        }

        // Right side must be an empty object `{}`
        let Expression::ObjectExpression(obj) = &logical.right else {
            return;
        };

        if obj.properties.is_empty() {
            ctx.report_warning(
                "no-useless-fallback-in-spread",
                "The empty object fallback in spread is unnecessary; spreading `undefined`/`null` is a no-op",
                Span::new(spread.span.start, spread.span.end),
            );
        }
    }
}

/// Unwrap parenthesized expressions.
fn unwrap_parens<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(paren) => unwrap_parens(&paren.expression),
        _ => expr,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessFallbackInSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_or_empty_object() {
        let diags = lint("var x = {...(obj || {})};");
        assert_eq!(diags.len(), 1, "...(obj || {{}}) should be flagged");
    }

    #[test]
    fn test_flags_coalesce_empty_object() {
        let diags = lint("var x = {...(obj ?? {})};");
        assert_eq!(diags.len(), 1, "...(obj ?? {{}}) should be flagged");
    }

    #[test]
    fn test_allows_spread_without_fallback() {
        let diags = lint("var x = {...obj};");
        assert!(
            diags.is_empty(),
            "spread without fallback should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_empty_fallback() {
        let diags = lint("var x = {...(obj || { a: 1 })};");
        assert!(diags.is_empty(), "non-empty fallback should not be flagged");
    }
}
