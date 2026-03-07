//! Rule: `typescript/no-empty-object-type`
//!
//! Disallow empty object type `{}` in type annotations. The empty object type
//! `{}` means "any non-nullish value" which is almost never what the developer
//! intends. Use `object`, `Record<string, unknown>`, or a more specific type
//! instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags empty object type literals (`{}`) used as type annotations.
#[derive(Debug)]
pub struct NoEmptyObjectType;

impl LintRule for NoEmptyObjectType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-empty-object-type".to_owned(),
            description: "Disallow empty object type `{}`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeLiteral(lit) = node else {
            return;
        };

        if !lit.members.is_empty() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/no-empty-object-type".to_owned(),
            message: "Empty object type `{}` is equivalent to any non-nullish value — use `object` or a more specific type instead".to_owned(),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some("Replace `{}` with `object`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace `{}` with `object`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement: "object".to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEmptyObjectType)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_object_type_annotation() {
        let diags = lint("let x: {} = y;");
        assert_eq!(
            diags.len(),
            1,
            "empty object type annotation should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_object_type_in_param() {
        let diags = lint("function f(x: {}) {}");
        assert_eq!(
            diags.len(),
            1,
            "empty object type in parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_object_type_with_members() {
        let diags = lint("let x: { a: number } = y;");
        assert!(
            diags.is_empty(),
            "object type with members should not be flagged"
        );
    }

    #[test]
    fn test_allows_object_keyword_type() {
        let diags = lint("let x: object = y;");
        assert!(
            diags.is_empty(),
            "object keyword type should not be flagged"
        );
    }
}
