//! Rule: `vitest/warn-todo`
//!
//! Warn when `test.todo` or `it.todo` is used. Todo tests are placeholders
//! for tests that need to be written. While useful during development, they
//! should not remain indefinitely in the test suite.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/warn-todo";

/// Warn when `test.todo` or `it.todo` is used.
#[derive(Debug)]
pub struct WarnTodo;

impl NativeRule for WarnTodo {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when `test.todo` or `it.todo` is used".to_owned(),
            category: Category::Suggestion,
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

        // Match `test.todo(...)` or `it.todo(...)`.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "todo" {
            return;
        }

        let obj_name = match &member.object {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if obj_name != "test" && obj_name != "it" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: format!("`{obj_name}.todo` found — implement or remove this test placeholder"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: Some(Fix {
                message: "Replace `.todo` with `.skip`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(member.property.span.start, member.property.span.end),
                    replacement: "skip".to_owned(),
                }],
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(WarnTodo)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_test_todo() {
        let source = r#"test.todo("implement this");"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`test.todo` should be flagged");
    }

    #[test]
    fn test_flags_it_todo() {
        let source = r#"it.todo("implement this");"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`it.todo` should be flagged");
    }

    #[test]
    fn test_allows_regular_test() {
        let source = r#"test("my test", () => { expect(1).toBe(1); });"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "regular test without `.todo` should not be flagged"
        );
    }
}
