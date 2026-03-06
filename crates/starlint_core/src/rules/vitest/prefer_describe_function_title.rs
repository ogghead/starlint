//! Rule: `vitest/prefer-describe-function-title`
//!
//! Suggest that `describe` block titles reference the function being tested.
//! When a `describe` block wraps tests for a specific function, its title
//! should match the function name for discoverability and organization.
//! This rule flags `describe` calls where the title is an empty string.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/prefer-describe-function-title";

/// Suggest meaningful `describe` block titles.
#[derive(Debug)]
pub struct PreferDescribeFunctionTitle;

impl NativeRule for PreferDescribeFunctionTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce meaningful `describe` block titles".to_owned(),
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

        // Match `describe(...)` calls.
        let is_describe = match &call.callee {
            Expression::Identifier(id) => id.name.as_str() == "describe",
            _ => false,
        };

        if !is_describe {
            return;
        }

        // Check the first argument (the title).
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Flag empty string titles.
        if let Argument::StringLiteral(lit) = first_arg {
            if lit.value.as_str().trim().is_empty() {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "`describe` block should have a meaningful title — use the function name or feature being tested".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }

        // Flag template literals with no expressions that are empty.
        if let Argument::TemplateLiteral(tpl) = first_arg {
            if tpl.expressions.is_empty() {
                let is_empty = tpl
                    .quasis
                    .iter()
                    .all(|q| q.value.raw.as_str().trim().is_empty());
                if is_empty {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "`describe` block should have a meaningful title — use the function name or feature being tested".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDescribeFunctionTitle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_describe_title() {
        let source = r#"describe("", () => {});"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`describe` with empty title should be flagged"
        );
    }

    #[test]
    fn test_allows_meaningful_describe_title() {
        let source = r#"describe("calculateTotal", () => {});"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`describe` with meaningful title should not be flagged"
        );
    }

    #[test]
    fn test_flags_whitespace_only_describe_title() {
        let source = r#"describe("  ", () => {});"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "`describe` with whitespace-only title should be flagged"
        );
    }
}
