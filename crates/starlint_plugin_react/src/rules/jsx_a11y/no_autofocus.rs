//! Rule: `jsx-a11y/no-autofocus`
//!
//! Forbid `autoFocus` attribute.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-autofocus";

#[derive(Debug)]
pub struct NoAutofocus;

impl LintRule for NoAutofocus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `autoFocus` attribute".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        let autofocus_span = opening.attributes.iter().find_map(|attr_id| {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                (attr.name.as_str() == "autoFocus")
                    .then(|| Span::new(attr.span.start, attr.span.end))
            } else {
                None
            }
        });

        if let Some(attr_span) = autofocus_span {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not use `autoFocus`. It can reduce usability and accessibility for users"
                        .to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Remove `autoFocus` attribute".to_owned(),
                    edits: vec![Edit {
                        span: attr_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAutofocus)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_autofocus() {
        let diags = lint(r"const el = <input autoFocus />;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_without_autofocus() {
        let diags = lint(r"const el = <input />;");
        assert!(diags.is_empty());
    }
}
