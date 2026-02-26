//! Rule: `no-useless-computed-key`
//!
//! Disallow unnecessary computed property keys in objects and classes.
//! `{["foo"]: 1}` is equivalent to `{foo: 1}` and the computed form
//! is unnecessarily complex.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags computed property keys that use a literal value unnecessarily.
#[derive(Debug)]
pub struct NoUselessComputedKey;

impl NativeRule for NoUselessComputedKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-computed-key".to_owned(),
            description: "Disallow unnecessary computed property keys".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::ObjectProperty(prop) => {
                if prop.computed && is_literal_key(&prop.key) {
                    ctx.report_warning(
                        "no-useless-computed-key",
                        "Unnecessary computed property key — use a literal key instead",
                        Span::new(prop.span.start, prop.span.end),
                    );
                }
            }
            AstKind::MethodDefinition(method) => {
                if method.computed && is_literal_key(&method.key) {
                    ctx.report_warning(
                        "no-useless-computed-key",
                        "Unnecessary computed property key — use a literal key instead",
                        Span::new(method.span.start, method.span.end),
                    );
                }
            }
            AstKind::PropertyDefinition(prop) => {
                if prop.computed && is_literal_key(&prop.key) {
                    ctx.report_warning(
                        "no-useless-computed-key",
                        "Unnecessary computed property key — use a literal key instead",
                        Span::new(prop.span.start, prop.span.end),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Check if a property key is a literal string or number.
const fn is_literal_key(key: &PropertyKey<'_>) -> bool {
    matches!(
        key,
        PropertyKey::StringLiteral(_) | PropertyKey::NumericLiteral(_)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessComputedKey)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_computed_key() {
        let diags = lint("var obj = { [\"foo\"]: 1 };");
        assert_eq!(
            diags.len(),
            1,
            "computed string key should be flagged"
        );
    }

    #[test]
    fn test_allows_variable_computed_key() {
        let diags = lint("var obj = { [foo]: 1 };");
        assert!(
            diags.is_empty(),
            "computed variable key should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_key() {
        let diags = lint("var obj = { foo: 1 };");
        assert!(
            diags.is_empty(),
            "regular key should not be flagged"
        );
    }

    #[test]
    fn test_flags_number_computed_key() {
        let diags = lint("var obj = { [0]: 1 };");
        assert_eq!(
            diags.len(),
            1,
            "computed number key should be flagged"
        );
    }
}
