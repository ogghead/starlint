//! Rule: `typescript/no-implied-eval`
//!
//! Disallow implied `eval()` usage. Flags calls to `setTimeout` and
//! `setInterval` where the first argument is a string literal (which gets
//! evaluated as code), and `new Function()` with a string literal body.
//!
//! Simplified syntax-only version — full checking requires type information.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-implied-eval";

/// Functions that perform implied eval when given a string argument.
const TIMER_FUNCTIONS: &[&str] = &["setTimeout", "setInterval"];

/// Flags `setTimeout(string)`, `setInterval(string)`, and `new Function(string)`.
#[derive(Debug)]
pub struct NoImpliedEval;

impl NativeRule for NoImpliedEval {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow implied `eval()` usage".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression, AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::CallExpression(call) => {
                let callee_name = match &call.callee {
                    Expression::Identifier(id) => Some(id.name.as_str()),
                    Expression::StaticMemberExpression(member) => {
                        // Handle `window.setTimeout(...)` and `globalThis.setInterval(...)`
                        let is_global_object = matches!(
                            &member.object,
                            Expression::Identifier(id)
                                if id.name.as_str() == "window"
                                    || id.name.as_str() == "globalThis"
                        );
                        is_global_object.then(|| member.property.name.as_str())
                    }
                    _ => None,
                };

                let Some(name) = callee_name else {
                    return;
                };

                if !TIMER_FUNCTIONS.contains(&name) {
                    return;
                }

                // Flag if the first argument is a string literal.
                if call
                    .arguments
                    .first()
                    .is_some_and(|arg| matches!(arg, Argument::StringLiteral(_)))
                {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "Implied `eval()` — do not pass a string to `{name}()`, use a function instead"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstKind::NewExpression(new_expr) => {
                // Flag `new Function("string body")`
                let is_function_constructor = matches!(
                    &new_expr.callee,
                    Expression::Identifier(id) if id.name.as_str() == "Function"
                );

                if is_function_constructor
                    && new_expr
                        .arguments
                        .last()
                        .is_some_and(|arg| matches!(arg, Argument::StringLiteral(_)))
                {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Implied `eval()` — do not use the `Function` constructor with a string body".to_owned(),
                        span: Span::new(new_expr.span.start, new_expr.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImpliedEval)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_set_timeout_with_string() {
        let diags = lint("setTimeout(\"alert('hi')\", 100);");
        assert_eq!(
            diags.len(),
            1,
            "setTimeout with string arg should be flagged"
        );
    }

    #[test]
    fn test_flags_set_interval_with_string() {
        let diags = lint("setInterval(\"doStuff()\", 1000);");
        assert_eq!(
            diags.len(),
            1,
            "setInterval with string arg should be flagged"
        );
    }

    #[test]
    fn test_flags_new_function_with_string() {
        let diags = lint("var f = new Function(\"return 1\");");
        assert_eq!(
            diags.len(),
            1,
            "new Function with string arg should be flagged"
        );
    }

    #[test]
    fn test_allows_set_timeout_with_function() {
        let diags = lint("setTimeout(() => {}, 100);");
        assert!(
            diags.is_empty(),
            "setTimeout with function arg should not be flagged"
        );
    }

    #[test]
    fn test_allows_set_interval_with_function() {
        let diags = lint("setInterval(function() {}, 1000);");
        assert!(
            diags.is_empty(),
            "setInterval with function arg should not be flagged"
        );
    }

    #[test]
    fn test_flags_window_set_timeout_with_string() {
        let diags = lint("window.setTimeout(\"alert('hi')\", 100);");
        assert_eq!(
            diags.len(),
            1,
            "window.setTimeout with string arg should be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_call() {
        let diags = lint("console.log(\"hello\");");
        assert!(
            diags.is_empty(),
            "unrelated call with string arg should not be flagged"
        );
    }
}
