//! Rule: `prefer-dom-node-append`
//!
//! Prefer `parent.append(child)` over `parent.appendChild(child)`.
//! `.append()` accepts multiple arguments, accepts strings, and does not
//! return the appended node — making it more flexible for common use cases.

#![allow(clippy::or_fun_call)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `appendChild()` calls, suggesting `.append()` instead.
#[derive(Debug)]
pub struct PreferDomNodeAppend;

impl LintRule for PreferDomNodeAppend {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-append".to_owned(),
            description: "Prefer `Node.append()` over `Node.appendChild()`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::map_unwrap_or
    )]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Callee must be a static member expression like `parent.appendChild(...)`
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        if member.property.as_str() != "appendChild" {
            return;
        }

        // Compute property span from source text
        let source = ctx.source_text();
        let call_text = source
            .get(call.span.start as usize..call.span.end as usize)
            .unwrap_or("");
        let prop_span = call_text.find("appendChild").map_or(
            Span::new(call.span.start, call.span.end),
            |offset| {
                let start = call.span.start.saturating_add(offset as u32);
                Span::new(start, start.saturating_add(12)) // "appendChild".len() == 12
            },
        );
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

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferDomNodeAppend)];
        lint_source(source, "test.js", &rules)
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
