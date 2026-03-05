//! Rule: `prefer-modern-dom-apis`
//!
//! Prefer modern DOM APIs over older ones. Flags `insertBefore`,
//! `replaceChild`, `removeChild`, and `appendChild` in favor of their
//! modern replacements: `before`/`after`, `replaceWith`, `remove`, and `append`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags legacy DOM mutation methods in favor of modern alternatives.
#[derive(Debug)]
pub struct PreferModernDomApis;

/// Legacy DOM mutation methods and their modern replacements.
const LEGACY_METHODS: &[(&str, &str)] = &[
    ("insertBefore", "before/after"),
    ("replaceChild", "replaceWith"),
    ("removeChild", "remove"),
    ("appendChild", "append"),
];

impl NativeRule for PreferModernDomApis {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-modern-dom-apis".to_owned(),
            description: "Prefer modern DOM APIs over legacy mutation methods".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();

        let Some((_legacy, modern)) = LEGACY_METHODS
            .iter()
            .find(|(legacy, _)| *legacy == method_name)
        else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-modern-dom-apis".to_owned(),
            message: format!("Prefer `{modern}` over `{method_name}`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Use `{modern}` instead of `{method_name}` for cleaner, more readable code"
            )),
            fix: None,
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferModernDomApis)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_insert_before() {
        let diags = lint("parent.insertBefore(newNode, refNode);");
        assert_eq!(diags.len(), 1, "insertBefore should be flagged");
    }

    #[test]
    fn test_flags_replace_child() {
        let diags = lint("parent.replaceChild(newNode, oldNode);");
        assert_eq!(diags.len(), 1, "replaceChild should be flagged");
    }

    #[test]
    fn test_flags_remove_child() {
        let diags = lint("parent.removeChild(child);");
        assert_eq!(diags.len(), 1, "removeChild should be flagged");
    }

    #[test]
    fn test_flags_append_child() {
        let diags = lint("parent.appendChild(child);");
        assert_eq!(diags.len(), 1, "appendChild should be flagged");
    }

    #[test]
    fn test_allows_remove() {
        let diags = lint("node.remove();");
        assert!(diags.is_empty(), "remove() should not be flagged");
    }

    #[test]
    fn test_allows_append() {
        let diags = lint("parent.append(child);");
        assert!(diags.is_empty(), "append() should not be flagged");
    }

    #[test]
    fn test_allows_before() {
        let diags = lint("node.before(newNode);");
        assert!(diags.is_empty(), "before() should not be flagged");
    }

    #[test]
    fn test_allows_replace_with() {
        let diags = lint("node.replaceWith(newNode);");
        assert!(diags.is_empty(), "replaceWith() should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("parent.contains(child);");
        assert!(
            diags.is_empty(),
            "unrelated DOM methods should not be flagged"
        );
    }
}
