//! Rule: `jest/no-jasmine-globals`
//!
//! Error when Jasmine globals like `jasmine.createSpy`, `spyOn`, `fail` are used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-jasmine-globals";

/// Standalone Jasmine global identifiers that should not be used.
const JASMINE_GLOBALS: &[&str] = &["spyOn", "spyOnProperty", "fail", "pending"];

/// Flags Jasmine-specific globals that should be replaced with Jest equivalents.
#[derive(Debug)]
pub struct NoJasmineGlobals;

impl NativeRule for NoJasmineGlobals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow Jasmine globals — use Jest equivalents".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        match &call.callee {
            // Direct calls to spyOn, fail, pending, etc.
            Expression::Identifier(id) if JASMINE_GLOBALS.contains(&id.name.as_str()) => {
                // Fix: `spyOn(x, y)` → `jest.spyOn(x, y)`
                let fix = (id.name.as_str() == "spyOn").then(|| {
                    let source = ctx.source_text();
                    let call_text = &source[call.span.start as usize..call.span.end as usize];
                    let replacement = format!("jest.{call_text}");
                    Fix {
                        message: format!("Replace with `jest.{}`", id.name),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`{}` is a Jasmine global — use the Jest equivalent instead",
                        id.name
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Replace `{}` with Jest equivalent", id.name)),
                    fix,
                    labels: vec![],
                });
            }
            // jasmine.createSpy(), jasmine.createSpyObj(), jasmine.any(), etc.
            Expression::StaticMemberExpression(member) => {
                let is_jasmine = matches!(
                    &member.object,
                    Expression::Identifier(id) if id.name.as_str() == "jasmine"
                );
                if is_jasmine {
                    // Fix: `jasmine.createSpy(...)` → `jest.fn()`
                    let fix = (member.property.name.as_str() == "createSpy").then(|| Fix {
                        message: "Replace with `jest.fn()`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: "jest.fn()".to_owned(),
                        }],
                        is_snippet: false,
                    });

                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "`jasmine.{}` is a Jasmine API — use Jest equivalents like `jest.fn()` instead",
                            member.property.name
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: Some("Use Jest equivalent".to_owned()),
                        fix,
                        labels: vec![],
                    });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoJasmineGlobals)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_spy_on() {
        let diags = lint("spyOn(obj, 'method');");
        assert_eq!(diags.len(), 1, "`spyOn` should be flagged");
    }

    #[test]
    fn test_flags_jasmine_create_spy() {
        let diags = lint("jasmine.createSpy('name');");
        assert_eq!(diags.len(), 1, "`jasmine.createSpy` should be flagged");
    }

    #[test]
    fn test_allows_jest_fn() {
        let diags = lint("jest.fn();");
        assert!(diags.is_empty(), "`jest.fn()` should not be flagged");
    }
}
