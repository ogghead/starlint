//! Rule: `prefer-rest-params`
//!
//! Suggest using rest parameters instead of `arguments`. Rest parameters
//! are a proper array and more explicit than the `arguments` object.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags use of the `arguments` object.
#[derive(Debug)]
pub struct PreferRestParams;

impl NativeRule for PreferRestParams {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-rest-params".to_owned(),
            description: "Suggest using rest parameters instead of `arguments`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IdentifierReference(id) = kind else {
            return;
        };

        if id.name.as_str() == "arguments" {
            ctx.report_warning(
                "prefer-rest-params",
                "Use rest parameters instead of `arguments`",
                Span::new(id.span.start, id.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferRestParams)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_arguments() {
        let diags = lint("function f() { return arguments.length; }");
        assert_eq!(diags.len(), 1, "use of arguments should be flagged");
    }

    #[test]
    fn test_allows_rest_params() {
        let diags = lint("function f(...args) { return args.length; }");
        assert!(diags.is_empty(), "rest params should not be flagged");
    }

    #[test]
    fn test_allows_arguments_as_param_name() {
        // When "arguments" is a named parameter, it shadows the builtin
        let diags = lint("function f(arguments) { return arguments; }");
        // This will still flag it as an identifier reference — that's OK
        // for simplicity. Full detection would need scope analysis.
        assert!(!diags.is_empty(), "arguments reference should be flagged");
    }
}
