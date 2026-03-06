//! Rule: `jest/no-focused-tests`
//!
//! Error when `fdescribe`, `fit`, `test.only`, `it.only`, `describe.only` are used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-focused-tests";

/// Focused-test prefixed identifiers.
const FOCUSED_IDENTIFIERS: &[&str] = &["fdescribe", "fit"];

/// Identifiers that can have `.only` called on them.
const ONLY_BASES: &[&str] = &["describe", "it", "test"];

/// Flags focused tests that would cause other tests to be skipped in CI.
#[derive(Debug)]
pub struct NoFocusedTests;

impl NativeRule for NoFocusedTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow focused tests (`.only`, `fdescribe`, `fit`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        match &call.callee {
            // fdescribe(...) or fit(...)
            Expression::Identifier(id) if FOCUSED_IDENTIFIERS.contains(&id.name.as_str()) => {
                let replacement = match id.name.as_str() {
                    "fdescribe" => "describe",
                    "fit" => "it",
                    _ => return,
                };
                let id_span = Span::new(id.span.start, id.span.end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Unexpected focused test: `{}()` will prevent other tests from running",
                        id.name
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Replace `{}` with `{replacement}`", id.name)),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: id_span,
                            replacement: replacement.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
            // describe.only(...), it.only(...), test.only(...)
            Expression::StaticMemberExpression(member) => {
                if member.property.name.as_str() == "only" {
                    let is_test_base = matches!(
                        &member.object,
                        Expression::Identifier(id) if ONLY_BASES.contains(&id.name.as_str())
                    );
                    if is_test_base {
                        let base_name = if let Expression::Identifier(id) = &member.object {
                            id.name.as_str()
                        } else {
                            "test"
                        };
                        // Replace `test.only` with `test` (remove `.only`)
                        let callee_span = Span::new(member.span().start, member.span().end);
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "Unexpected focused test: `{base_name}.only()` will prevent other tests from running"
                            ),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Error,
                            help: Some(format!("Remove `.only` from `{base_name}.only`")),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: format!("Replace `{base_name}.only` with `{base_name}`"),
                                edits: vec![Edit {
                                    span: callee_span,
                                    replacement: base_name.to_owned(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoFocusedTests)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_fdescribe() {
        let diags = lint("fdescribe('suite', () => {});");
        assert_eq!(diags.len(), 1, "`fdescribe` should be flagged");
    }

    #[test]
    fn test_flags_test_only() {
        let diags = lint("test.only('my test', () => {});");
        assert_eq!(diags.len(), 1, "`test.only` should be flagged");
    }

    #[test]
    fn test_allows_regular_test() {
        let diags = lint("test('my test', () => {});");
        assert!(diags.is_empty(), "regular `test()` should not be flagged");
    }
}
