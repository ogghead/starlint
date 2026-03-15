//! Rule: `nextjs/google-font-display`
//!
//! Enforce `display` parameter in Google Fonts URLs to avoid invisible text
//! during font loading (FOIT).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/google-font-display";

/// Flags Google Fonts URLs that are missing the `display` query parameter.
#[derive(Debug)]
pub struct GoogleFontDisplay;

impl LintRule for GoogleFontDisplay {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `display` parameter in Google Fonts URLs".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StringLiteral])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StringLiteral(lit) = node else {
            return;
        };

        let value = lit.value.as_str();

        if !value.contains("fonts.googleapis.com") {
            return;
        }

        // Check for the `display=` query parameter
        let has_display = value.contains("&display=") || value.contains("?display=");

        if !has_display {
            let separator = if value.contains('?') { "&" } else { "?" };
            let mut fixed_url = String::with_capacity(value.len().saturating_add(14));
            fixed_url.push_str(value);
            fixed_url.push_str(separator);
            fixed_url.push_str("display=swap");

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Google Fonts URL is missing the `display` parameter. Add `&display=swap` to avoid invisible text during loading".to_owned(),
                span: Span::new(lit.span.start, lit.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `display=swap` parameter".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(
                            lit.span.start.saturating_add(1),
                            lit.span.end.saturating_sub(1),
                        ),
                        replacement: fixed_url,
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

    starlint_rule_framework::lint_rule_test!(GoogleFontDisplay);

    #[test]
    fn test_flags_missing_display() {
        let diags = lint(r#"const url = "https://fonts.googleapis.com/css?family=Roboto";"#);
        assert_eq!(diags.len(), 1, "missing display param should be flagged");
    }

    #[test]
    fn test_allows_with_display() {
        let diags =
            lint(r#"const url = "https://fonts.googleapis.com/css?family=Roboto&display=swap";"#);
        assert!(diags.is_empty(), "URL with display param should pass");
    }

    #[test]
    fn test_ignores_non_google_fonts() {
        let diags = lint(r#"const url = "https://example.com/fonts";"#);
        assert!(
            diags.is_empty(),
            "non-Google Fonts URL should not be flagged"
        );
    }
}
