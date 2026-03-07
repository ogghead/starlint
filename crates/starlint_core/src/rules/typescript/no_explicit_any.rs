//! Rule: `typescript/no-explicit-any`
//!
//! Disallow the `any` type annotation. Using `any` disables `TypeScript` type
//! checking for the annotated binding, defeating the purpose of the type system.
//! Prefer `unknown`, generics, or explicit types instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags usage of the `any` type annotation.
#[derive(Debug)]
pub struct NoExplicitAny;

impl LintRule for NoExplicitAny {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-explicit-any".to_owned(),
            description: "Disallow the `any` type annotation".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSAnyKeyword])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSAnyKeyword(keyword) = node else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "typescript/no-explicit-any".to_owned(),
            message: "Unexpected `any` type annotation — use `unknown` or a specific type instead"
                .to_owned(),
            span: Span::new(keyword.span.start, keyword.span.end),
            severity: Severity::Warning,
            help: Some("Replace `any` with `unknown`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Replace with `unknown`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(keyword.span.start, keyword.span.end),
                    replacement: "unknown".to_owned(),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExplicitAny)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_any_variable() {
        let diags = lint("let x: any;");
        assert_eq!(diags.len(), 1, "`any` type annotation should be flagged");
    }

    #[test]
    fn test_flags_any_parameter() {
        let diags = lint("function f(x: any) {}");
        assert_eq!(
            diags.len(),
            1,
            "`any` in function parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_unknown() {
        let diags = lint("let x: unknown;");
        assert!(diags.is_empty(), "`unknown` should not be flagged");
    }

    #[test]
    fn test_allows_string() {
        let diags = lint("let x: string;");
        assert!(diags.is_empty(), "`string` should not be flagged");
    }
}
