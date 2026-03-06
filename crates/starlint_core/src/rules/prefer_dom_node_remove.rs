//! Rule: `prefer-dom-node-remove`
//!
//! Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`.
//! The `.remove()` method is simpler and supported in all modern browsers.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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

        if member.property.name.as_str() != "removeChild" {
            return;
        }

        // Extract the child argument to build `child.remove()`
        let call_span = Span::new(call.span.start, call.span.end);
        let fix = if call.arguments.len() == 1 {
            let Some(arg) = call.arguments.first() else {
                return;
            };
            let arg_span = arg.span();
            let child_text = ctx
                .source_text()
                .get(
                    usize::try_from(arg_span.start).unwrap_or(0)
                        ..usize::try_from(arg_span.end).unwrap_or(0),
                )
                .unwrap_or("")
                .to_owned();
            (!child_text.is_empty()).then(|| Fix {
                message: "Replace with `child.remove()`".to_owned(),
                edits: vec![Edit {
                    span: call_span,
                    replacement: format!("{child_text}.remove()"),
                }],
                is_snippet: false,
            })
        } else {
            None
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-dom-node-remove".to_owned(),
            message: "Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`"
                .to_owned(),
            span: call_span,
            severity: Severity::Warning,
            help: Some("Use `childNode.remove()` instead".to_owned()),
            fix,
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
