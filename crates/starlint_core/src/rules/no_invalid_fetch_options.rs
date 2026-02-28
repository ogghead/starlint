//! Rule: `no-invalid-fetch-options`
//!
//! Flag `fetch()` calls where the options object contains both a `body`
//! property and a `method` set to `"GET"` or `"HEAD"`. GET and HEAD
//! requests do not accept a body — including one is a bug that most
//! runtimes will either silently ignore or reject.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey, PropertyKind};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `fetch()` calls with `body` on GET/HEAD requests.
#[derive(Debug)]
pub struct NoInvalidFetchOptions;

/// HTTP methods that do not accept a request body.
const BODYLESS_METHODS: &[&str] = &["GET", "HEAD"];

impl NativeRule for NoInvalidFetchOptions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-invalid-fetch-options".to_owned(),
            description: "Disallow `body` in `fetch()` options for GET/HEAD requests".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `fetch(...)` call
        if !is_fetch_call(&call.callee) {
            return;
        }

        // Need at least two arguments: url and options
        let Some(second_arg) = call.arguments.get(1) else {
            return;
        };

        // Options must be an object literal
        let Argument::ObjectExpression(options) = second_arg else {
            return;
        };

        let mut has_body = false;
        let mut method_value: Option<String> = None;

        for prop_kind in &options.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else {
                continue;
            };

            // Only check init properties (not getters/setters)
            if prop.kind != PropertyKind::Init {
                continue;
            }

            let Some(key_name) = static_key_name(&prop.key) else {
                continue;
            };

            if key_name == "body" {
                has_body = true;
            } else if key_name == "method" {
                method_value = string_literal_value(&prop.value);
            }
        }

        if has_body {
            if let Some(method) = method_value {
                let upper_method = method.to_uppercase();
                if BODYLESS_METHODS.contains(&upper_method.as_str()) {
                    ctx.report_error(
                        "no-invalid-fetch-options",
                        &format!(
                            "`fetch()` with method `{method}` should not have a `body` — {upper_method} requests do not accept a body"
                        ),
                        Span::new(call.span.start, call.span.end),
                    );
                }
            }
        }
    }
}

/// Check if a call expression's callee is `fetch`.
fn is_fetch_call(callee: &Expression<'_>) -> bool {
    matches!(
        callee,
        Expression::Identifier(id) if id.name.as_str() == "fetch"
    )
}

/// Extract a static key name from a property key (identifier or string literal).
fn static_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

/// Extract the value of a string literal expression.
fn string_literal_value(expr: &Expression<'_>) -> Option<String> {
    if let Expression::StringLiteral(lit) = expr {
        Some(lit.value.to_string())
    } else {
        None
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInvalidFetchOptions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_get_with_body() {
        let diags = lint("fetch(url, { method: 'GET', body: data });");
        assert_eq!(diags.len(), 1, "GET with body should be flagged");
    }

    #[test]
    fn test_flags_head_with_body() {
        let diags = lint("fetch(url, { method: 'HEAD', body: data });");
        assert_eq!(diags.len(), 1, "HEAD with body should be flagged");
    }

    #[test]
    fn test_flags_get_lowercase_with_body() {
        let diags = lint("fetch(url, { method: 'get', body: data });");
        assert_eq!(diags.len(), 1, "lowercase get with body should be flagged");
    }

    #[test]
    fn test_allows_post_with_body() {
        let diags = lint("fetch(url, { method: 'POST', body: data });");
        assert!(diags.is_empty(), "POST with body should not be flagged");
    }

    #[test]
    fn test_allows_put_with_body() {
        let diags = lint("fetch(url, { method: 'PUT', body: data });");
        assert!(diags.is_empty(), "PUT with body should not be flagged");
    }

    #[test]
    fn test_allows_fetch_no_options() {
        let diags = lint("fetch(url);");
        assert!(
            diags.is_empty(),
            "fetch with no options should not be flagged"
        );
    }

    #[test]
    fn test_allows_get_no_body() {
        let diags = lint("fetch(url, { method: 'GET' });");
        assert!(diags.is_empty(), "GET without body should not be flagged");
    }

    #[test]
    fn test_allows_body_no_method() {
        // Default method is GET, but without an explicit method property
        // we don't flag it — the user may have a reason, and we only
        // flag the explicit mismatch.
        let diags = lint("fetch(url, { body: data });");
        assert!(
            diags.is_empty(),
            "body without explicit method should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_fetch_call() {
        let diags = lint("request(url, { method: 'GET', body: data });");
        assert!(diags.is_empty(), "non-fetch calls should not be flagged");
    }

    #[test]
    fn test_allows_variable_options() {
        let diags = lint("fetch(url, options);");
        assert!(diags.is_empty(), "variable options should not be flagged");
    }
}
