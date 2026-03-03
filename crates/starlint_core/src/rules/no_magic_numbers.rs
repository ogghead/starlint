//! Rule: `no-magic-numbers`
//!
//! Flag magic numbers -- numeric literals used directly instead of named
//! constants. Common values like 0, 1, -1, and 2 are allowed.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags numeric literals that are not in the allowed set `{0, 1, -1, 2}`.
#[derive(Debug)]
pub struct NoMagicNumbers;

/// Check if a float value is in the set of allowed non-magic numbers.
#[allow(clippy::float_cmp)]
fn is_allowed_value(value: f64) -> bool {
    value == 0.0 || value == 1.0 || value == -1.0 || value == 2.0
}

impl NativeRule for NoMagicNumbers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-magic-numbers".to_owned(),
            description: "Disallow magic numbers".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NumericLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NumericLiteral(lit) = kind else {
            return;
        };

        if is_allowed_value(lit.value) {
            return;
        }

        ctx.report_warning(
            "no-magic-numbers",
            &format!("No magic number: `{}`", lit.raw_str()),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMagicNumbers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_magic_number() {
        let diags = lint("const x = 42;");
        assert_eq!(diags.len(), 1, "42 is a magic number and should be flagged");
    }

    #[test]
    fn test_allows_zero() {
        let diags = lint("const x = 0;");
        assert!(diags.is_empty(), "0 should not be flagged");
    }

    #[test]
    fn test_allows_one() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "1 should not be flagged");
    }

    #[test]
    fn test_allows_two() {
        let diags = lint("const x = 2;");
        assert!(diags.is_empty(), "2 should not be flagged");
    }

    #[test]
    fn test_flags_ten_in_loop() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert_eq!(diags.len(), 1, "10 should be flagged as a magic number");
    }

    #[test]
    fn test_flags_large_number() {
        let diags = lint("const timeout = 3000;");
        assert_eq!(diags.len(), 1, "3000 should be flagged as a magic number");
    }
}
