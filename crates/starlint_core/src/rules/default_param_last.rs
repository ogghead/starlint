//! Rule: `default-param-last`
//!
//! Enforce default parameters to be last. Non-default parameters after
//! a default parameter cannot take advantage of defaults without passing
//! `undefined` explicitly.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags default parameters that are not in the last positions.
#[derive(Debug)]
pub struct DefaultParamLast;

impl NativeRule for DefaultParamLast {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "default-param-last".to_owned(),
            description: "Enforce default parameters to be last".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let params = match kind {
            AstKind::Function(f) => &f.params,
            AstKind::ArrowFunctionExpression(arrow) => &arrow.params,
            _ => return,
        };

        // Find the last non-default, non-rest parameter.
        // Any default parameter before it is a violation.
        let mut seen_non_default = false;
        for param in params.items.iter().rev() {
            if param.initializer.is_some() {
                // This is a default parameter
                if seen_non_default {
                    ctx.report_warning(
                        "default-param-last",
                        "Default parameters should be last",
                        Span::new(param.span.start, param.span.end),
                    );
                }
            } else {
                seen_non_default = true;
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DefaultParamLast)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_defaults_at_end() {
        let diags = lint("function foo(a, b = 1) {}");
        assert!(diags.is_empty(), "default at end should not be flagged");
    }

    #[test]
    fn test_flags_default_before_non_default() {
        let diags = lint("function foo(a = 1, b) {}");
        assert_eq!(
            diags.len(),
            1,
            "default before non-default should be flagged"
        );
    }

    #[test]
    fn test_allows_all_defaults() {
        let diags = lint("function foo(a = 1, b = 2) {}");
        assert!(diags.is_empty(), "all defaults should not be flagged");
    }

    #[test]
    fn test_allows_no_defaults() {
        let diags = lint("function foo(a, b) {}");
        assert!(diags.is_empty(), "no defaults should not be flagged");
    }

    #[test]
    fn test_flags_arrow_function() {
        let diags = lint("const foo = (a = 1, b) => {};");
        assert_eq!(
            diags.len(),
            1,
            "arrow with default before non-default should be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_violations() {
        let diags = lint("function foo(a = 1, b = 2, c) {}");
        assert_eq!(
            diags.len(),
            2,
            "multiple defaults before non-default should all be flagged"
        );
    }
}
