//! Rule: `jest/prefer-to-have-length`
//!
//! Suggest `expect(arr).toHaveLength(n)` over `expect(arr.length).toBe(n)`.
//! The `toHaveLength` matcher provides clearer failure messages that include
//! the actual length.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `expect(arr.length).toBe(n)` patterns.
#[derive(Debug)]
pub struct PreferToHaveLength;

impl NativeRule for PreferToHaveLength {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-to-have-length".to_owned(),
            description:
                "Suggest using `toHaveLength()` instead of checking `.length` with `toBe()`"
                    .to_owned(),
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

        // Must be `.toBe(...)` or `.toEqual(...)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let method = member.property.name.as_str();
        if method != "toBe" && method != "toEqual" {
            return;
        }

        // Object must be `expect(...)` call
        let Expression::CallExpression(expect_call) = &member.object else {
            return;
        };
        let is_expect = matches!(
            &expect_call.callee,
            Expression::Identifier(id) if id.name.as_str() == "expect"
        );
        if !is_expect {
            return;
        }

        // First arg of expect() must be `something.length`
        let Some(first_arg) = expect_call.arguments.first() else {
            return;
        };
        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };
        let Expression::StaticMemberExpression(arg_member) = arg_expr else {
            return;
        };
        if arg_member.property.name.as_str() != "length" {
            return;
        }

        // Two edits:
        // 1. Replace `arr.length` inside expect() with just `arr` (the object of the .length member)
        // 2. Replace the matcher name (`toBe`/`toEqual`) with `toHaveLength`
        let source = ctx.source_text();
        let obj_span = arg_member.object.span();
        let obj_text = source
            .get(
                usize::try_from(obj_span.start).unwrap_or(0)
                    ..usize::try_from(obj_span.end).unwrap_or(0),
            )
            .unwrap_or("");
        let arg_full_span = Span::new(arg_member.span().start, arg_member.span().end);
        let matcher_span = Span::new(member.property.span.start, member.property.span.end);

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-to-have-length".to_owned(),
            message: "Use `toHaveLength()` instead of asserting on `.length` with `toBe()`"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `expect(arr).toHaveLength(n)`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace with `toHaveLength`".to_owned(),
                edits: vec![
                    Edit {
                        span: arg_full_span,
                        replacement: obj_text.to_owned(),
                    },
                    Edit {
                        span: matcher_span,
                        replacement: "toHaveLength".to_owned(),
                    },
                ],
                is_snippet: false,
            }),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToHaveLength)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_length_to_be() {
        let diags = lint("expect(arr.length).toBe(3);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.length).toBe(3)` should be flagged"
        );
    }

    #[test]
    fn test_flags_length_to_equal() {
        let diags = lint("expect(arr.length).toEqual(0);");
        assert_eq!(
            diags.len(),
            1,
            "`expect(arr.length).toEqual(0)` should be flagged"
        );
    }

    #[test]
    fn test_allows_to_have_length() {
        let diags = lint("expect(arr).toHaveLength(3);");
        assert!(diags.is_empty(), "`toHaveLength()` should not be flagged");
    }
}
