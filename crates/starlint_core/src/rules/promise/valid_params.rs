//! Rule: `promise/valid-params`
//!
//! Enforce correct number of parameters to Promise static methods.
//! `Promise.resolve()` takes 0-1 args, `Promise.reject()` takes 0-1 args,
//! `Promise.all()` takes exactly 1 arg, etc.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Promise.all()`, `Promise.race()`, `Promise.allSettled()`,
/// and `Promise.any()` called with incorrect argument counts.
#[derive(Debug)]
pub struct ValidParams;

impl NativeRule for ValidParams {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/valid-params".to_owned(),
            description: "Enforce correct number of params to Promise methods".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let Expression::Identifier(ident) = &member.object else {
            return;
        };

        if ident.name.as_str() != "Promise" {
            return;
        }

        let method = member.property.name.as_str();
        let arg_count = call.arguments.len();

        let expected = match method {
            // These require exactly 1 argument (an iterable)
            "all" | "allSettled" | "any" | "race" => Some((1, 1)),
            // These take 0 or 1 argument
            "resolve" | "reject" => Some((0, 1)),
            _ => None,
        };

        if let Some((min, max)) = expected {
            if arg_count < min || arg_count > max {
                let expected_msg = if min == max {
                    format!("exactly {min}")
                } else {
                    format!("{min}-{max}")
                };
                ctx.report(Diagnostic {
                    rule_name: "promise/valid-params".to_owned(),
                    message: format!(
                        "`Promise.{method}()` expects {expected_msg} argument(s), got {arg_count}"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ValidParams)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_promise_all_no_args() {
        let diags = lint("Promise.all();");
        assert_eq!(diags.len(), 1, "should flag Promise.all() with no args");
    }

    #[test]
    fn test_flags_promise_resolve_two_args() {
        let diags = lint("Promise.resolve(1, 2);");
        assert_eq!(diags.len(), 1, "should flag Promise.resolve with 2 args");
    }

    #[test]
    fn test_allows_promise_all_one_arg() {
        let diags = lint("Promise.all([p1, p2]);");
        assert!(diags.is_empty(), "Promise.all with 1 arg should be allowed");
    }
}
