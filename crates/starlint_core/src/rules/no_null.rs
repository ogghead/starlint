//! Rule: `no-null` (unicorn)
//!
//! Disallow the use of `null`. Prefer `undefined` for consistency.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags use of `null`.
#[derive(Debug)]
pub struct NoNull;

impl NativeRule for NoNull {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-null".to_owned(),
            description: "Disallow the use of `null`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NullLiteral(lit) = kind else {
            return;
        };

        ctx.report_warning(
            "no-null",
            "Avoid using `null` — prefer `undefined` for consistency",
            Span::new(lit.span.start, lit.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNull)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_null() {
        let diags = lint("var x = null;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_undefined() {
        let diags = lint("var x = undefined;");
        assert!(diags.is_empty());
    }
}
