//! Rule: `jest/prefer-strict-equal`
//!
//! Suggest `toStrictEqual` over `toEqual`. `toStrictEqual` checks that
//! objects have the same type and structure, unlike `toEqual` which performs
//! a more lenient recursive comparison that ignores `undefined` properties.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.toEqual()` calls that could use `.toStrictEqual()`.
#[derive(Debug)]
pub struct PreferStrictEqual;

impl NativeRule for PreferStrictEqual {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-strict-equal".to_owned(),
            description: "Suggest using `toStrictEqual()` over `toEqual()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name.as_str() != "toEqual" {
            return;
        }

        if !is_expect_chain(&member.object) {
            return;
        }

        let prop_span = Span::new(member.property.span.start, member.property.span.end);
        ctx.report(Diagnostic {
            rule_name: "jest/prefer-strict-equal".to_owned(),
            message: "Use `toStrictEqual()` instead of `toEqual()` for stricter equality checking"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace `toEqual` with `toStrictEqual`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `toStrictEqual`".to_owned(),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: "toStrictEqual".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Check if an expression is an `expect(...)` call or a chain like
/// `expect(...).not`.
fn is_expect_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(id) if id.name.as_str() == "expect")
        }
        Expression::StaticMemberExpression(member) => is_expect_chain(&member.object),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStrictEqual)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_equal() {
        let diags = lint("expect(result).toEqual({ a: 1 });");
        assert_eq!(
            diags.len(),
            1,
            "`toEqual` should be flagged in favor of `toStrictEqual`"
        );
    }

    #[test]
    fn test_flags_to_equal_with_not() {
        let diags = lint("expect(result).not.toEqual({ a: 1 });");
        assert_eq!(diags.len(), 1, "`.not.toEqual` should also be flagged");
    }

    #[test]
    fn test_allows_to_strict_equal() {
        let diags = lint("expect(result).toStrictEqual({ a: 1 });");
        assert!(diags.is_empty(), "`toStrictEqual` should not be flagged");
    }
}
