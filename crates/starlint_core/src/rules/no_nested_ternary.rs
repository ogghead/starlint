//! Rule: `no-nested-ternary`
//!
//! Disallow nested ternary expressions. Nested ternaries are difficult to
//! read and should be refactored into `if`/`else` statements or extracted
//! into separate variables.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags ternary expressions that contain nested ternary sub-expressions.
#[derive(Debug)]
pub struct NoNestedTernary;

impl NativeRule for NoNestedTernary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-nested-ternary".to_owned(),
            description: "Disallow nested ternary expressions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ConditionalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ConditionalExpression(expr) = kind else {
            return;
        };

        let nested_in_consequent = matches!(&expr.consequent, Expression::ConditionalExpression(_));
        let nested_in_alternate = matches!(&expr.alternate, Expression::ConditionalExpression(_));

        if !nested_in_consequent && !nested_in_alternate {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-nested-ternary".to_owned(),
            message: "Nested ternary expression".to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some(
                "Refactor into if/else statements or extract into separate variables".to_owned(),
            ),
            fix: None,
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNestedTernary)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_nested_in_consequent() {
        let diags = lint("const x = a ? b ? 1 : 2 : 3;");
        assert_eq!(diags.len(), 1, "should flag nested ternary in consequent");
    }

    #[test]
    fn test_flags_nested_in_alternate() {
        let diags = lint("const x = a ? 1 : b ? 2 : 3;");
        assert_eq!(diags.len(), 1, "should flag nested ternary in alternate");
    }

    #[test]
    fn test_allows_simple_ternary() {
        let diags = lint("const x = a ? 1 : 2;");
        assert!(diags.is_empty(), "simple ternary should not be flagged");
    }

    #[test]
    fn test_allows_non_ternary() {
        let diags = lint("const x = a || b;");
        assert!(diags.is_empty(), "logical expression should not be flagged");
    }
}
