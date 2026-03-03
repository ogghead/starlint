//! Rule: `vitest/prefer-to-be-truthy`
//!
//! Suggest `toBeTruthy()` over `toBe(true)`. The `toBeTruthy()` matcher is
//! more idiomatic in Vitest for checking truthy values and provides clearer
//! intent.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-to-be-truthy";

/// Suggest `toBeTruthy()` over `toBe(true)`.
#[derive(Debug)]
pub struct PreferToBeTruthy;

impl NativeRule for PreferToBeTruthy {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `toBeTruthy()` over `toBe(true)`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Match `.toBe(true)`.
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "toBe" {
            return;
        }

        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_true = matches!(first_arg, Argument::BooleanLiteral(lit) if lit.value);

        if is_true {
            ctx.report_warning(
                RULE_NAME,
                "Prefer `toBeTruthy()` over `toBe(true)`",
                Span::new(call.span.start, call.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferToBeTruthy)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_be_true() {
        let source = "expect(value).toBe(true);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`.toBe(true)` should be flagged");
    }

    #[test]
    fn test_allows_to_be_truthy() {
        let source = "expect(value).toBeTruthy();";
        let diags = lint(source);
        assert!(diags.is_empty(), "`.toBeTruthy()` should not be flagged");
    }

    #[test]
    fn test_allows_to_be_false() {
        let source = "expect(value).toBe(false);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`.toBe(false)` should not be flagged by this rule"
        );
    }
}
