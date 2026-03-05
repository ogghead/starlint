//! Rule: `promise/no-native`
//!
//! Forbid use of the native `Promise` global. Useful when a project
//! requires a polyfill (e.g. `bluebird`) for consistency or extra features.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags any reference to the `Promise` identifier.
#[derive(Debug)]
pub struct NoNative;

impl NativeRule for NoNative {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-native".to_owned(),
            description: "Forbid native `Promise` (enforce polyfill)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::IdentifierReference])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::IdentifierReference(ident) = kind else {
            return;
        };

        if ident.name.as_str() == "Promise" {
            ctx.report(Diagnostic {
                rule_name: "promise/no-native".to_owned(),
                message: "Avoid using native `Promise` — use the configured polyfill instead"
                    .to_owned(),
                span: Span::new(ident.span.start, ident.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNative)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_promise_resolve() {
        let diags = lint("const p = Promise.resolve(1);");
        assert!(!diags.is_empty(), "should flag native Promise usage");
    }

    #[test]
    fn test_flags_new_promise() {
        let diags = lint("const p = new Promise((r) => r(1));");
        assert!(!diags.is_empty(), "should flag new Promise");
    }

    #[test]
    fn test_allows_non_promise() {
        let diags = lint("const m = new Map();");
        assert!(diags.is_empty(), "non-Promise should not be flagged");
    }
}
