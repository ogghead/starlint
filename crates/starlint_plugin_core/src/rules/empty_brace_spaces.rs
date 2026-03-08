//! Rule: `empty-brace-spaces`
//!
//! Disallow spaces inside empty object braces. `{ }` should be `{}`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags object expressions with spaces inside empty braces like `{ }`.
#[derive(Debug)]
pub struct EmptyBraceSpaces;

impl LintRule for EmptyBraceSpaces {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "empty-brace-spaces".to_owned(),
            description: "Disallow spaces inside empty object braces".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ObjectExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ObjectExpression(obj) = node else {
            return;
        };

        // Only flag empty objects (no properties, no spread).
        if !obj.properties.is_empty() {
            return;
        }

        let Some(raw) = ctx
            .source_text()
            .get(obj.span.start as usize..obj.span.end as usize)
        else {
            return;
        };

        // If the source text is already `{}`, no issue.
        if raw == "{}" {
            return;
        }

        // Check for spaces/whitespace between the braces.
        let inner = &raw[1..raw.len().saturating_sub(1)];
        if !inner.trim().is_empty() {
            // Something other than whitespace between braces — not our concern.
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "empty-brace-spaces".to_owned(),
            message: "Unexpected spaces inside empty braces".to_owned(),
            span: Span::new(obj.span.start, obj.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `{}`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove spaces inside braces".to_owned(),
                edits: vec![Edit {
                    span: Span::new(obj.span.start, obj.span.end),
                    replacement: "{}".to_owned(),
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
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EmptyBraceSpaces)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_space_in_empty_braces() {
        let diags = lint("const x = { };");
        assert_eq!(diags.len(), 1, "should flag empty braces with space");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("{}"),
            "fix should remove spaces"
        );
    }

    #[test]
    fn test_flags_multiple_spaces() {
        let diags = lint("const x = {   };");
        assert_eq!(
            diags.len(),
            1,
            "should flag empty braces with multiple spaces"
        );
    }

    #[test]
    fn test_allows_empty_no_spaces() {
        let diags = lint("const x = {};");
        assert!(
            diags.is_empty(),
            "empty braces without spaces should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_empty_object() {
        let diags = lint("const x = { a: 1 };");
        assert!(diags.is_empty(), "non-empty object should not be flagged");
    }
}
