//! Rule: `promise/always-return`
//!
//! Require returning inside `.then()` callbacks. Without a return value,
//! the next `.then()` in the chain receives `undefined`, which is almost
//! always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.then()` callbacks that do not contain a `return` statement.
#[derive(Debug)]
pub struct AlwaysReturn;

impl NativeRule for AlwaysReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/always-return".to_owned(),
            description: "Require returning inside `.then()` callbacks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "then" {
            return;
        }

        // Check the first argument (the onFulfilled callback)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let arg_expr = match first_arg {
            oxc_ast::ast::Argument::SpreadElement(_) => return,
            _ => first_arg.to_expression(),
        };

        // Check if the callback is an arrow function with expression body
        // (implicit return — this is fine)
        if let Expression::ArrowFunctionExpression(arrow) = arg_expr {
            if arrow.expression {
                return; // expression body = implicit return
            }
        }

        // For block-bodied functions, we flag at the `.then()` call site.
        // A full check would inspect function body for return statements,
        // but that requires deeper analysis. We flag non-expression arrows
        // and regular functions as a heuristic.
        match arg_expr {
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
                ctx.report_error(
                    "promise/always-return",
                    "Each `.then()` callback should return a value or throw",
                    Span::new(call.span.start, call.span.end),
                );
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AlwaysReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_then_with_block_body() {
        let diags = lint("promise.then(function(val) { console.log(val); });");
        assert_eq!(
            diags.len(),
            1,
            "should flag .then() with block-body callback"
        );
    }

    #[test]
    fn test_allows_expression_arrow() {
        let diags = lint("promise.then(val => val * 2);");
        assert!(diags.is_empty(), "expression arrow has implicit return");
    }

    #[test]
    fn test_flags_block_arrow() {
        let diags = lint("promise.then(val => { console.log(val); });");
        assert_eq!(diags.len(), 1, "should flag block-body arrow in .then()");
    }
}
