//! Rule: `prefer-native-coercion-functions`
//!
//! Prefer passing native coercion functions like `Number`, `String`, or
//! `Boolean` directly instead of wrapping them in arrow functions.
//! `x => Number(x)` is equivalent to just `Number` and adds unnecessary
//! indirection.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, BindingPattern, Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Coercion function names that can be passed directly.
const COERCION_FUNCTIONS: &[&str] = &["Number", "String", "Boolean"];

/// Flags arrow functions that simply wrap a native coercion call.
#[derive(Debug)]
pub struct PreferNativeCoercionFunctions;

impl NativeRule for PreferNativeCoercionFunctions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-native-coercion-functions".to_owned(),
            description:
                "Prefer passing `Number`, `String`, or `Boolean` directly instead of wrapping in an arrow function"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrowFunctionExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ArrowFunctionExpression(arrow) = kind else {
            return;
        };

        // Must have exactly one parameter
        if arrow.params.items.len() != 1 {
            return;
        }

        // Must be an expression body (not a block body)
        if !arrow.expression {
            return;
        }

        // The parameter must be a simple binding identifier (not destructured)
        let Some(param) = arrow.params.items.first() else {
            return;
        };
        let BindingPattern::BindingIdentifier(param_id) = &param.pattern else {
            return;
        };
        let param_name = param_id.name.as_str();

        // Body must have exactly one statement (the expression statement)
        let Some(stmt) = arrow.body.statements.first() else {
            return;
        };
        let Statement::ExpressionStatement(expr_stmt) = stmt else {
            return;
        };

        // The expression must be a call to a coercion function
        let Expression::CallExpression(call) = &expr_stmt.expression else {
            return;
        };

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // Callee must be an identifier that is a coercion function
        let Expression::Identifier(callee_id) = &call.callee else {
            return;
        };
        let callee_name = callee_id.name.as_str();
        if !COERCION_FUNCTIONS.contains(&callee_name) {
            return;
        }

        // The single argument must be an identifier reference matching the parameter
        let Some(Argument::Identifier(arg_id)) = call.arguments.first() else {
            return;
        };

        if arg_id.name.as_str() != param_name {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-native-coercion-functions".to_owned(),
            message: format!(
                "Unnecessary arrow function wrapper — pass `{callee_name}` directly"
            ),
            span: Span::new(arrow.span.start, arrow.span.end),
            severity: Severity::Warning,
            help: Some(format!("Replace with `{callee_name}`")),
            fix: Some(Fix {
                message: format!("Replace with `{callee_name}`"),
                edits: vec![Edit {
                    span: Span::new(arrow.span.start, arrow.span.end),
                    replacement: callee_name.to_owned(),
                }],
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferNativeCoercionFunctions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_number_wrapper() {
        let diags = lint("arr.map(x => Number(x));");
        assert_eq!(diags.len(), 1, "x => Number(x) should be flagged");
    }

    #[test]
    fn test_flags_string_wrapper() {
        let diags = lint("arr.map(x => String(x));");
        assert_eq!(diags.len(), 1, "x => String(x) should be flagged");
    }

    #[test]
    fn test_flags_boolean_wrapper() {
        let diags = lint("arr.map(x => Boolean(x));");
        assert_eq!(diags.len(), 1, "x => Boolean(x) should be flagged");
    }

    #[test]
    fn test_allows_parse_int_wrapper() {
        let diags = lint("arr.map(x => parseInt(x));");
        assert!(diags.is_empty(), "x => parseInt(x) should not be flagged");
    }

    #[test]
    fn test_allows_direct_coercion() {
        let diags = lint("arr.map(Number);");
        assert!(
            diags.is_empty(),
            "direct Number reference should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_param_name() {
        let diags = lint("arr.map(x => Number(y));");
        assert!(
            diags.is_empty(),
            "different argument name should not be flagged"
        );
    }

    #[test]
    fn test_allows_multiple_params() {
        let diags = lint("arr.map((x, i) => Number(x));");
        assert!(
            diags.is_empty(),
            "arrow with multiple params should not be flagged"
        );
    }

    #[test]
    fn test_allows_block_body() {
        let diags = lint("arr.map(x => { return Number(x); });");
        assert!(diags.is_empty(), "block body arrow should not be flagged");
    }

    #[test]
    fn test_allows_coercion_with_extra_args() {
        let diags = lint("arr.map(x => Number(x, 10));");
        assert!(
            diags.is_empty(),
            "coercion call with extra args should not be flagged"
        );
    }
}
