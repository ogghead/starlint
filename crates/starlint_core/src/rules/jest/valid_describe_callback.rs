//! Rule: `jest/valid-describe-callback`
//!
//! Error when `describe` callback is async or returns a value.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/valid-describe-callback";

/// Flags `describe` blocks with async callbacks or return values.
#[derive(Debug)]
pub struct ValidDescribeCallback;

impl NativeRule for ValidDescribeCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow async `describe` callbacks and return values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check callee is `describe`
        let is_describe = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "describe"
        );

        if !is_describe {
            return;
        }

        // The callback is the second argument
        let Some(callback) = call.arguments.get(1) else {
            return;
        };

        match callback {
            Argument::ArrowFunctionExpression(arrow) => {
                if arrow.r#async {
                    ctx.report_error(
                        RULE_NAME,
                        "`describe` callback must not be async",
                        Span::new(arrow.span.start, arrow.span.end),
                    );
                }
                // Check for expression body (implicit return)
                if arrow.expression {
                    ctx.report_error(
                        RULE_NAME,
                        "`describe` callback must not return a value",
                        Span::new(arrow.span.start, arrow.span.end),
                    );
                }
            }
            Argument::FunctionExpression(func) => {
                if func.r#async {
                    ctx.report_error(
                        RULE_NAME,
                        "`describe` callback must not be async",
                        Span::new(func.span.start, func.span.end),
                    );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ValidDescribeCallback)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_async_describe() {
        let diags = lint("describe('suite', async () => {});");
        assert_eq!(diags.len(), 1, "async describe callback should be flagged");
    }

    #[test]
    fn test_flags_async_function_describe() {
        let diags = lint("describe('suite', async function() {});");
        assert_eq!(
            diags.len(),
            1,
            "async function describe callback should be flagged"
        );
    }

    #[test]
    fn test_allows_sync_describe() {
        let diags = lint("describe('suite', () => { it('works', () => {}); });");
        assert!(
            diags.is_empty(),
            "sync describe callback should not be flagged"
        );
    }
}
