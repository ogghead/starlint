//! Rule: `typescript/unbound-method`
//!
//! Disallow referencing unbound methods. Flags passing methods as callbacks
//! without binding — for example, `array.forEach(obj.method)` where
//! `obj.method` is a member expression used as an argument instead of being
//! called directly.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This heuristic checks call expression arguments for member expressions
//! that are not themselves called (i.e. passed as bare references).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/unbound-method";

/// Common callback-accepting methods where passing an unbound method reference
/// is likely a bug.
const CALLBACK_METHODS: &[&str] = &[
    "forEach",
    "map",
    "filter",
    "some",
    "every",
    "find",
    "findIndex",
    "reduce",
    "flatMap",
    "sort",
    "then",
    "catch",
];

/// Flags member expressions passed as callback arguments to common higher-order
/// functions, which likely lose their `this` binding.
#[derive(Debug)]
pub struct UnboundMethod;

impl NativeRule for UnboundMethod {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow referencing unbound methods as callbacks".to_owned(),
            category: Category::Correctness,
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

        // Only flag callback-accepting methods (e.g. `.forEach(...)`, `.map(...)`)
        let method_name = match &call.callee {
            Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
            _ => None,
        };

        let Some(method) = method_name else {
            return;
        };

        if !CALLBACK_METHODS.contains(&method) {
            return;
        }

        // Check each argument for bare member expressions
        for arg in &call.arguments {
            let Some(arg_expr) = arg.as_expression() else {
                continue;
            };

            let member_span = match arg_expr {
                Expression::StaticMemberExpression(member) => Some(member.span),
                Expression::ComputedMemberExpression(member) => Some(member.span),
                _ => None,
            };

            if let Some(span) = member_span {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Avoid passing an unbound method reference to `.{method}()` — \
                         the method will lose its `this` context; use an arrow function \
                         or `.bind()` instead"
                    ),
                    span: Span::new(span.start, span.end),
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UnboundMethod)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_member_expression_callback() {
        let diags = lint("const arr = [1, 2, 3]; arr.forEach(obj.handler);");
        assert_eq!(
            diags.len(),
            1,
            "passing obj.handler as callback should be flagged"
        );
    }

    #[test]
    fn test_flags_map_with_member_expression() {
        let diags = lint("const items = ['a', 'b']; items.map(converter.transform);");
        assert_eq!(
            diags.len(),
            1,
            "passing converter.transform to map should be flagged"
        );
    }

    #[test]
    fn test_allows_arrow_function_callback() {
        let diags = lint("const arr = [1, 2]; arr.forEach(x => obj.handler(x));");
        assert!(
            diags.is_empty(),
            "arrow function wrapping the method should not be flagged"
        );
    }

    #[test]
    fn test_allows_bound_method() {
        let diags = lint("const arr = [1, 2]; arr.forEach(obj.handler.bind(obj));");
        assert!(diags.is_empty(), "bound method should not be flagged");
    }

    #[test]
    fn test_allows_identifier_callback() {
        let diags = lint("const arr = [1, 2]; arr.forEach(myFunction);");
        assert!(
            diags.is_empty(),
            "plain identifier callback should not be flagged"
        );
    }
}
