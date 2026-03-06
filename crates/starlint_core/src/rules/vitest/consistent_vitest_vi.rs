//! Rule: `vitest/consistent-vitest-vi`
//!
//! Enforce consistent usage of `vi` instead of `vitest` for mock functions.
//! The `vi` shorthand is the idiomatic way to access Vitest's mock utilities.
//! Using `vitest.fn()`, `vitest.mock()`, or `vitest.spyOn()` should be
//! replaced with `vi.fn()`, `vi.mock()`, or `vi.spyOn()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/consistent-vitest-vi";

/// Methods on the `vitest` object that should use `vi` instead.
const VI_METHODS: &[&str] = &[
    "fn",
    "mock",
    "spyOn",
    "hoisted",
    "unmock",
    "doMock",
    "doUnmock",
    "importActual",
    "importMock",
    "restoreAllMocks",
    "resetAllMocks",
    "clearAllMocks",
    "useFakeTimers",
    "useRealTimers",
];

/// Enforce using `vi` shorthand instead of `vitest` for mock utilities.
#[derive(Debug)]
pub struct ConsistentVitestVi;

impl NativeRule for ConsistentVitestVi {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce using `vi` instead of `vitest` for mock utilities".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
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

        // Check if the object is `vitest`.
        let Expression::Identifier(obj) = &member.object else {
            return;
        };

        if obj.name.as_str() != "vitest" {
            return;
        }

        let method_name = member.property.name.as_str();

        if VI_METHODS.contains(&method_name) {
            // Replace the `vitest` identifier with `vi` in the object position
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Use `vi.{method_name}()` instead of `vitest.{method_name}()`"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "Replace `vitest.{method_name}` with `vi.{method_name}`"
                )),
                fix: Some(Fix {
                    message: "Replace `vitest` with `vi`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(obj.span.start, obj.span.end),
                        replacement: "vi".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentVitestVi)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_vitest_fn() {
        let source = "const mock = vitest.fn();";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`vitest.fn()` should be flagged");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("vi.fn")),
            "message should suggest `vi.fn()`"
        );
    }

    #[test]
    fn test_flags_vitest_mock() {
        let source = r#"vitest.mock("./module");"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`vitest.mock()` should be flagged");
    }

    #[test]
    fn test_allows_vi_fn() {
        let source = "const mock = vi.fn();";
        let diags = lint(source);
        assert!(diags.is_empty(), "`vi.fn()` should not be flagged");
    }
}
