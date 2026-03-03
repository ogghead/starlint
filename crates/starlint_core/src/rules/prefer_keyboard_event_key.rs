//! Rule: `prefer-keyboard-event-key`
//!
//! Prefer `KeyboardEvent.key` over the deprecated `keyCode`, `charCode`, and
//! `which` properties. The `key` property provides a human-readable string
//! and is supported in all modern browsers.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Deprecated `KeyboardEvent` properties that should be replaced with `key`.
const DEPRECATED_PROPERTIES: &[&str] = &["keyCode", "charCode", "which"];

/// Flags access to deprecated `KeyboardEvent` properties.
#[derive(Debug)]
pub struct PreferKeyboardEventKey;

impl NativeRule for PreferKeyboardEventKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-keyboard-event-key".to_owned(),
            description:
                "Prefer `KeyboardEvent.key` over deprecated `keyCode`, `charCode`, and `which`"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StaticMemberExpression(member) = kind else {
            return;
        };

        let prop = member.property.name.as_str();
        if !DEPRECATED_PROPERTIES.contains(&prop) {
            return;
        }

        ctx.report_warning(
            "prefer-keyboard-event-key",
            &format!("Use `KeyboardEvent.key` instead of deprecated `{prop}`"),
            Span::new(member.span.start, member.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferKeyboardEventKey)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_key_code() {
        let diags = lint("var code = event.keyCode;");
        assert_eq!(diags.len(), 1, "event.keyCode should be flagged");
    }

    #[test]
    fn test_flags_char_code() {
        let diags = lint("var code = e.charCode;");
        assert_eq!(diags.len(), 1, "e.charCode should be flagged");
    }

    #[test]
    fn test_flags_which() {
        let diags = lint("var code = e.which;");
        assert_eq!(diags.len(), 1, "e.which should be flagged");
    }

    #[test]
    fn test_flags_any_object() {
        let diags = lint("var code = obj.keyCode;");
        assert_eq!(diags.len(), 1, "obj.keyCode should be flagged");
    }

    #[test]
    fn test_allows_key() {
        let diags = lint("var k = event.key;");
        assert!(diags.is_empty(), "event.key should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_property() {
        let diags = lint("var v = event.target;");
        assert!(diags.is_empty(), "event.target should not be flagged");
    }
}
