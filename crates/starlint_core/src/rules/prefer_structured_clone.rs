//! Rule: `prefer-structured-clone` (unicorn)
//!
//! Prefer `structuredClone()` over `JSON.parse(JSON.stringify())` for
//! deep cloning objects. `structuredClone` is more efficient and handles
//! more data types correctly.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `JSON.parse(JSON.stringify(x))` patterns.
#[derive(Debug)]
pub struct PreferStructuredClone;

impl NativeRule for PreferStructuredClone {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-structured-clone".to_owned(),
            description: "Prefer structuredClone over JSON.parse(JSON.stringify())".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for JSON.parse(...)
        if !is_json_method_call(&call.callee, "parse") {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // The argument must be JSON.stringify(...)
        let Some(arg) = call.arguments.first() else {
            return;
        };

        let is_json_stringify = match arg {
            oxc_ast::ast::Argument::CallExpression(inner_call) => {
                is_json_method_call(&inner_call.callee, "stringify")
                    && inner_call.arguments.len() == 1
            }
            _ => false,
        };

        if is_json_stringify {
            ctx.report_warning(
                "prefer-structured-clone",
                "Prefer `structuredClone(x)` over `JSON.parse(JSON.stringify(x))`",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if an expression is `JSON.methodName`.
fn is_json_method_call(expr: &Expression<'_>, method: &str) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };

    let Expression::Identifier(obj) = &member.object else {
        return false;
    };

    obj.name == "JSON" && member.property.name == method
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStructuredClone)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_json_parse_stringify() {
        let diags = lint("var copy = JSON.parse(JSON.stringify(obj));");
        assert_eq!(
            diags.len(),
            1,
            "JSON.parse(JSON.stringify()) should be flagged"
        );
    }

    #[test]
    fn test_allows_structured_clone() {
        let diags = lint("var copy = structuredClone(obj);");
        assert!(diags.is_empty(), "structuredClone should not be flagged");
    }

    #[test]
    fn test_allows_json_parse_alone() {
        let diags = lint("var data = JSON.parse(text);");
        assert!(diags.is_empty(), "JSON.parse alone should not be flagged");
    }

    #[test]
    fn test_allows_json_stringify_alone() {
        let diags = lint("var text = JSON.stringify(obj);");
        assert!(
            diags.is_empty(),
            "JSON.stringify alone should not be flagged"
        );
    }
}
