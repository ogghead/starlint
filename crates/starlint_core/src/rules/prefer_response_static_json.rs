//! Rule: `prefer-response-static-json`
//!
//! Prefer `Response.json()` over `new Response(JSON.stringify())`.
//! The static `Response.json()` method is cleaner and automatically
//! sets the `Content-Type` header to `application/json`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Response(JSON.stringify(...))` patterns.
#[derive(Debug)]
pub struct PreferResponseStaticJson;

impl NativeRule for PreferResponseStaticJson {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-response-static-json".to_owned(),
            description: "Prefer `Response.json()` over `new Response(JSON.stringify())`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Check that the constructor is `Response`.
        let Expression::Identifier(callee_id) = &new_expr.callee else {
            return;
        };

        if callee_id.name.as_str() != "Response" {
            return;
        }

        // Check the first argument is a call to `JSON.stringify()`.
        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        let is_json_stringify = match first_arg {
            oxc_ast::ast::Argument::CallExpression(call) => is_json_stringify_call(&call.callee),
            _ => false,
        };

        if is_json_stringify {
            ctx.report(Diagnostic {
                rule_name: "prefer-response-static-json".to_owned(),
                message: "Prefer `Response.json()` over `new Response(JSON.stringify())`"
                    .to_owned(),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: Some(
                    "Use `Response.json(data)` — it is cleaner and sets Content-Type automatically"
                        .to_owned(),
                ),
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is `JSON.stringify`.
fn is_json_stringify_call(callee: &Expression<'_>) -> bool {
    matches!(
        callee,
        Expression::StaticMemberExpression(member)
            if member.property.name.as_str() == "stringify"
            && matches!(
                &member.object,
                Expression::Identifier(id) if id.name.as_str() == "JSON"
            )
    )
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferResponseStaticJson)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_response_json_stringify() {
        let diags = lint("new Response(JSON.stringify(data));");
        assert_eq!(
            diags.len(),
            1,
            "new Response(JSON.stringify()) should be flagged"
        );
    }

    #[test]
    fn test_flags_with_options() {
        let diags = lint("new Response(JSON.stringify(data), { status: 200 });");
        assert_eq!(
            diags.len(),
            1,
            "new Response(JSON.stringify(), options) should be flagged"
        );
    }

    #[test]
    fn test_allows_plain_string() {
        let diags = lint("new Response('hello');");
        assert!(
            diags.is_empty(),
            "new Response with a plain string should not be flagged"
        );
    }

    #[test]
    fn test_allows_response_json() {
        let diags = lint("Response.json(data);");
        assert!(diags.is_empty(), "Response.json() should not be flagged");
    }

    #[test]
    fn test_allows_new_response_no_args() {
        let diags = lint("new Response();");
        assert!(
            diags.is_empty(),
            "new Response() with no args should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_constructor() {
        let diags = lint("new Foo(JSON.stringify(data));");
        assert!(
            diags.is_empty(),
            "non-Response constructor should not be flagged"
        );
    }
}
