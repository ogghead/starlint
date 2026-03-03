//! Rule: `prefer-dom-node-remove`
//!
//! Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`.
//! The `.remove()` method is simpler and supported in all modern browsers.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `removeChild()` calls, suggesting `.remove()` instead.
#[derive(Debug)]
pub struct PreferDomNodeRemove;

impl NativeRule for PreferDomNodeRemove {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-remove".to_owned(),
            description: "Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
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

        if member.property.name.as_str() != "removeChild" {
            return;
        }

        ctx.report_warning(
            "prefer-dom-node-remove",
            "Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`",
            Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDomNodeRemove)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_remove_child() {
        let diags = lint("parent.removeChild(child);");
        assert_eq!(
            diags.len(),
            1,
            "parent.removeChild(child) should be flagged"
        );
    }

    #[test]
    fn test_flags_list_remove_child() {
        let diags = lint("list.removeChild(item);");
        assert_eq!(diags.len(), 1, "list.removeChild(item) should be flagged");
    }

    #[test]
    fn test_allows_remove() {
        let diags = lint("child.remove();");
        assert!(diags.is_empty(), "child.remove() should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("parent.appendChild(child);");
        assert!(
            diags.is_empty(),
            "parent.appendChild(child) should not be flagged"
        );
    }
}
