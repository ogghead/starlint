//! Rule: `jest/valid-title`
//!
//! Warn when `describe`/`it`/`test` titles are empty strings or not string literals.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/valid-title";

/// Test block names to check.
const TEST_BLOCKS: &[&str] = &["describe", "it", "test"];

/// Flags `describe`/`it`/`test` calls with empty or non-string titles.
#[derive(Debug)]
pub struct ValidTitle;

impl NativeRule for ValidTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require valid titles for `describe`/`it`/`test` blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is describe/it/test (direct identifier)
        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if !TEST_BLOCKS.contains(&callee_name) {
            return;
        }

        // Check the first argument (the title)
        let Some(first_arg) = call.arguments.first() else {
            ctx.report_warning(
                RULE_NAME,
                &format!("`{callee_name}()` must have a title as its first argument"),
                Span::new(call.span.start, call.span.end),
            );
            return;
        };

        match first_arg {
            Argument::StringLiteral(lit) => {
                if lit.value.is_empty() {
                    ctx.report_warning(
                        RULE_NAME,
                        &format!("`{callee_name}()` title must not be empty"),
                        Span::new(lit.span.start, lit.span.end),
                    );
                }
            }
            Argument::TemplateLiteral(_) => {
                // Template literals are acceptable
            }
            _ => {
                ctx.report_warning(
                    RULE_NAME,
                    &format!("`{callee_name}()` title must be a string literal"),
                    Span::new(call.span.start, call.span.end),
                );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ValidTitle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_title() {
        let diags = lint("describe('', () => {});");
        assert_eq!(diags.len(), 1, "empty describe title should be flagged");
    }

    #[test]
    fn test_flags_non_string_title() {
        let diags = lint("it(123, () => {});");
        assert_eq!(diags.len(), 1, "non-string title should be flagged");
    }

    #[test]
    fn test_allows_valid_title() {
        let diags = lint("test('should work', () => {});");
        assert!(diags.is_empty(), "valid string title should not be flagged");
    }
}
