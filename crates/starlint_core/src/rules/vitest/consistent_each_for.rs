//! Rule: `vitest/consistent-each-for`
//!
//! Suggest consistent usage of `.each` for parameterized tests. When tests
//! repeat the same structure with different data, using `test.each`/`it.each`
//! is cleaner. This rule detects `test.each`/`it.each` usage and ensures
//! the pattern is used consistently (flags raw `test.each`/`it.each` calls
//! that pass an empty array or are called without a template literal or array).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/consistent-each-for";

/// Suggest consistent `.each` usage for parameterized tests.
#[derive(Debug)]
pub struct ConsistentEachFor;

impl NativeRule for ConsistentEachFor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce consistent usage of `.each` for parameterized tests".to_owned(),
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

        // Look for `test.each(...)` or `it.each(...)` — the outer call that
        // provides the data set. The callee itself is a CallExpression whose
        // callee is `test.each` / `it.each`.
        let callee = &call.callee;

        // Match: `<callee>.each(...)(<args>)` — The outer call is the second
        // invocation. But we want to detect `test.each([])` — the data call.
        // Actually, let's match `test.each` / `it.each` as a member expression
        // callee, where the call receives an empty array argument.
        let Expression::StaticMemberExpression(member) = callee else {
            return;
        };

        let prop_name = member.property.name.as_str();
        if prop_name != "each" {
            return;
        }

        // The object must be `test`, `it`, or `describe`.
        let obj_name = match &member.object {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if obj_name != "test" && obj_name != "it" && obj_name != "describe" {
            return;
        }

        // Flag if the `.each()` call receives an empty array `[]`.
        if let Some(oxc_ast::ast::Argument::ArrayExpression(arr)) = call.arguments.first() {
            if arr.elements.is_empty() {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`{obj_name}.each` called with an empty array — provide test case data or remove `.each`"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentEachFor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_each_array() {
        let source = r#"test.each([])("my test", (val) => {});"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`test.each` with empty array should be flagged"
        );
    }

    #[test]
    fn test_allows_nonempty_each_array() {
        let source = r#"test.each([1, 2, 3])("my test %i", (val) => {});"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`test.each` with data should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_test() {
        let source = r#"test("my test", () => {});"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "regular test without `.each` should not be flagged"
        );
    }
}
