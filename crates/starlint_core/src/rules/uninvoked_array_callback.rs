//! Rule: `uninvoked-array-callback` (OXC)
//!
//! Detect passing a function reference to an array method that expects a
//! different number of arguments. For example, `['1', '2'].map(parseInt)`
//! doesn't work as expected because `parseInt` receives the index as the
//! second argument (radix).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Known dangerous combinations of array methods + function references.
const DANGEROUS_CALLBACKS: &[(&str, &str)] = &[
    ("map", "parseInt"),
    ("map", "parseFloat"),
    ("map", "Number"),
    ("forEach", "alert"),
    ("map", "Boolean"),
];

/// Flags potentially problematic function references passed to array methods.
#[derive(Debug)]
pub struct UninvokedArrayCallback;

impl NativeRule for UninvokedArrayCallback {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "uninvoked-array-callback".to_owned(),
            description: "Detect problematic function references in array callbacks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Get the method name from a member expression
        let method_name = match &call.callee {
            Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
            _ => None,
        };

        let Some(method) = method_name else {
            return;
        };

        // Check if the first argument is a known dangerous callback
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let Some(arg_expr) = first_arg.as_expression() else {
            return;
        };

        let callback_name = match arg_expr {
            Expression::Identifier(id) => Some(id.name.as_str()),
            _ => None,
        };

        let Some(cb_name) = callback_name else {
            return;
        };

        for &(arr_method, func_name) in DANGEROUS_CALLBACKS {
            if method == arr_method && cb_name == func_name {
                ctx.report_warning(
                    "uninvoked-array-callback",
                    &format!(
                        "Passing `{func_name}` directly to `.{arr_method}()` may produce \
                         unexpected results — the callback receives extra arguments (index, array)"
                    ),
                    Span::new(call.span.start, call.span.end),
                );
                return;
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UninvokedArrayCallback)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_map_parse_int() {
        let diags = lint("['1', '2', '3'].map(parseInt);");
        assert_eq!(diags.len(), 1, "map(parseInt) should be flagged");
    }

    #[test]
    fn test_allows_map_with_arrow() {
        let diags = lint("['1', '2', '3'].map(x => parseInt(x, 10));");
        assert!(
            diags.is_empty(),
            "map with arrow function should not be flagged"
        );
    }

    #[test]
    fn test_allows_map_with_custom_function() {
        let diags = lint("[1, 2, 3].map(double);");
        assert!(
            diags.is_empty(),
            "map with custom function should not be flagged"
        );
    }
}
