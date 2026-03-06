//! Rule: `jest/no-disabled-tests`
//!
//! Warn when `xdescribe`, `xit`, `xtest`, `test.skip`, `it.skip`, `describe.skip` are used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-disabled-tests";

/// Disabled-test prefixed identifiers.
const DISABLED_IDENTIFIERS: &[&str] = &["xdescribe", "xit", "xtest"];

/// Identifiers that can have `.skip` called on them.
const SKIP_BASES: &[&str] = &["describe", "it", "test"];

/// Flags disabled/skipped tests that may be forgotten.
#[derive(Debug)]
pub struct NoDisabledTests;

impl NativeRule for NoDisabledTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow disabled tests (`xdescribe`, `xtest`, `.skip`)".to_owned(),
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

        match &call.callee {
            // xdescribe(...), xit(...), xtest(...)
            Expression::Identifier(id) if DISABLED_IDENTIFIERS.contains(&id.name.as_str()) => {
                let replacement = match id.name.as_str() {
                    "xdescribe" => "describe",
                    "xit" => "it",
                    "xtest" => "test",
                    _ => return,
                };
                let id_span = Span::new(id.span.start, id.span.end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Unexpected disabled test: `{}()` — remove or re-enable",
                        id.name
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
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
            // describe.skip(...), it.skip(...), test.skip(...)
            Expression::StaticMemberExpression(member) => {
                if member.property.name.as_str() == "skip" {
                    let is_test_base = matches!(
                        &member.object,
                        Expression::Identifier(id) if SKIP_BASES.contains(&id.name.as_str())
                    );
                    if is_test_base {
                        let base_name = if let Expression::Identifier(id) = &member.object {
                            id.name.as_str()
                        } else {
                            "test"
                        };
                        // Replace `test.skip` with `test` (remove `.skip`)
                        let callee_span = Span::new(member.span().start, member.span().end);
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "Unexpected disabled test: `{base_name}.skip()` — remove or re-enable"
                            ),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Warning,
                            help: Some(format!("Remove `.skip` from `{base_name}.skip`")),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: format!("Replace `{base_name}.skip` with `{base_name}`"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDisabledTests)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_xtest() {
        let diags = lint("xtest('my test', () => {});");
        assert_eq!(diags.len(), 1, "`xtest` should be flagged");
    }

    #[test]
    fn test_flags_it_skip() {
        let diags = lint("it.skip('my test', () => {});");
        assert_eq!(diags.len(), 1, "`it.skip` should be flagged");
    }

    #[test]
    fn test_allows_regular_it() {
        let diags = lint("it('my test', () => {});");
        assert!(diags.is_empty(), "regular `it()` should not be flagged");
    }
}
