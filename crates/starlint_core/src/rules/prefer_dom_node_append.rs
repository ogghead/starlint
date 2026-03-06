//! Rule: `prefer-dom-node-append`
//!
//! Prefer `parent.append(child)` over `parent.appendChild(child)`.
//! `.append()` accepts multiple arguments, accepts strings, and does not
//! return the appended node — making it more flexible for common use cases.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `appendChild()` calls, suggesting `.append()` instead.
#[derive(Debug)]
pub struct PreferDomNodeAppend;

impl NativeRule for PreferDomNodeAppend {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-append".to_owned(),
            description: "Prefer `Node.append()` over `Node.appendChild()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Callee must be a static member expression like `parent.appendChild(...)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "appendChild" {
            return;
        }

        let prop_span = Span::new(member.property.span.start, member.property.span.end);
        ctx.report(Diagnostic {
            rule_name: "prefer-dom-node-append".to_owned(),
            message: "Prefer `Node.append()` over `Node.appendChild()`".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Replace `appendChild` with `append`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace `appendChild` with `append`".to_owned(),
                edits: vec![Edit {
                    span: prop_span,
                    replacement: "append".to_owned(),
                }],
                is_snippet: false,
            }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDomNodeAppend)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_append_child() {
        let diags = lint("parent.appendChild(child);");
        assert_eq!(diags.len(), 1, "appendChild should be flagged");
    }

    #[test]
    fn test_flags_list_append_child() {
        let diags = lint("list.appendChild(item);");
        assert_eq!(diags.len(), 1, "list.appendChild should be flagged");
    }

    #[test]
    fn test_allows_append() {
        let diags = lint("parent.append(child);");
        assert!(diags.is_empty(), "append() should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("parent.removeChild(child);");
        assert!(diags.is_empty(), "removeChild should not be flagged");
    }

    #[test]
    fn test_allows_standalone_function() {
        let diags = lint("appendChild(child);");
        assert!(
            diags.is_empty(),
            "standalone appendChild call should not be flagged"
        );
    }
}
