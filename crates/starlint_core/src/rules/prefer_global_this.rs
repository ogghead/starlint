//! Rule: `prefer-global-this` (unicorn)
//!
//! Prefer `globalThis` over `window`, `self`, or `global` for accessing
//! the global object. `globalThis` works in all environments.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Global object references that should be replaced with `globalThis`.
const DEPRECATED_GLOBALS: &[&str] = &["window", "self", "global"];

/// Flags references to `window`, `self`, or `global`.
#[derive(Debug)]
pub struct PreferGlobalThis;

impl NativeRule for PreferGlobalThis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-global-this".to_owned(),
            description: "Prefer globalThis over window/self/global".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IdentifierReference(ident) = kind else {
            return;
        };

        let name = ident.name.as_str();
        if DEPRECATED_GLOBALS.contains(&name) {
            ctx.report_warning(
                "prefer-global-this",
                &format!("Prefer `globalThis` over `{name}`"),
                Span::new(ident.span.start, ident.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferGlobalThis)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_window() {
        let diags = lint("var x = window.location;");
        assert_eq!(diags.len(), 1, "window should be flagged");
    }

    #[test]
    fn test_flags_global() {
        let diags = lint("var x = global.process;");
        assert_eq!(diags.len(), 1, "global should be flagged");
    }

    #[test]
    fn test_allows_global_this() {
        let diags = lint("var x = globalThis.location;");
        assert!(diags.is_empty(), "globalThis should not be flagged");
    }

    #[test]
    fn test_allows_other_identifiers() {
        let diags = lint("var x = foo.bar;");
        assert!(diags.is_empty(), "other identifiers should not be flagged");
    }
}
