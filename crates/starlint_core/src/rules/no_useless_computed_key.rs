//! Rule: `no-useless-computed-key`
//!
//! Disallow unnecessary computed property keys in objects and classes.
//! `{["foo"]: 1}` is equivalent to `{foo: 1}` and the computed form
//! is unnecessarily complex.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::MethodDefinition,
            AstType::ObjectProperty,
            AstType::PropertyDefinition,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let (computed, key, prop_span) = match kind {
            AstKind::ObjectProperty(prop) => (prop.computed, &prop.key, prop.span),
            AstKind::MethodDefinition(method) => (method.computed, &method.key, method.span),
            AstKind::PropertyDefinition(prop) => (prop.computed, &prop.key, prop.span),
            _ => return,
        };

        if !computed || !is_literal_key(key) {
            return;
        }

        let source = ctx.source_text();
        let key_span = key.span();
        let key_start = usize::try_from(key_span.start).unwrap_or(0);
        let key_end = usize::try_from(key_span.end).unwrap_or(0);
        let key_source = source.get(key_start..key_end).unwrap_or("");

        // Find [ before the key and ] after it in the source.
        let before = source.get(..key_start).unwrap_or("");
        let after = source.get(key_end..).unwrap_or("");
        let open = before.rfind('[').map(|p| u32::try_from(p).unwrap_or(0));
        let close = after
            .find(']')
            .map(|p| u32::try_from(key_end.saturating_add(p).saturating_add(1)).unwrap_or(0));

        let fix = if let (Some(open_pos), Some(close_pos)) = (open, close) {
            Some(Fix {
                message: "Remove computed brackets".to_owned(),
                edits: vec![Edit {
                    span: Span::new(open_pos, close_pos),
                    replacement: key_source.to_owned(),
                }],
                is_snippet: false,
            })
        } else {
            None
        };

        ctx.report(Diagnostic {
            rule_name: "no-useless-computed-key".to_owned(),
            message: "Unnecessary computed property key — use a literal key instead".to_owned(),
            span: Span::new(prop_span.start, prop_span.end),
            severity: Severity::Warning,
            help: Some("Remove the computed brackets".to_owned()),
            fix,
            labels: vec![],
        });
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
        assert_eq!(diags.len(), 1, "computed string key should be flagged");
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
        assert!(diags.is_empty(), "regular key should not be flagged");
    }

    #[test]
    fn test_flags_number_computed_key() {
        let diags = lint("var obj = { [0]: 1 };");
        assert_eq!(diags.len(), 1, "computed number key should be flagged");
    }
}
