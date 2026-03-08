//! Rule: `jsx-a11y/autocomplete-valid`
//!
//! Enforce `autocomplete` attribute has a valid value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/autocomplete-valid";

/// Valid autocomplete tokens (HTML spec).
const VALID_AUTOCOMPLETE: &[&str] = &[
    "on",
    "off",
    "name",
    "honorific-prefix",
    "given-name",
    "additional-name",
    "family-name",
    "honorific-suffix",
    "nickname",
    "email",
    "username",
    "new-password",
    "current-password",
    "one-time-code",
    "organization-title",
    "organization",
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
    "cc-name",
    "cc-given-name",
    "cc-additional-name",
    "cc-family-name",
    "cc-number",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-csc",
    "cc-type",
    "transaction-currency",
    "transaction-amount",
    "language",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "sex",
    "tel",
    "tel-country-code",
    "tel-national",
    "tel-area-code",
    "tel-local",
    "tel-extension",
    "impp",
    "url",
    "photo",
];

/// Elements that support the `autocomplete` attribute.
const AUTOCOMPLETE_ELEMENTS: &[&str] = &["input", "select", "textarea"];

#[derive(Debug)]
pub struct AutocompleteValid;

impl LintRule for AutocompleteValid {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `autocomplete` attribute has a valid value".to_owned(),
            category: Category::Correctness,
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

        // opening.name is a String
        let element_name = opening.name.as_str();

        if !AUTOCOMPLETE_ELEMENTS.contains(&element_name) {
            return;
        }

        for attr_id in &*opening.attributes {
            if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                let is_autocomplete = attr.name.as_str() == "autoComplete";

                if !is_autocomplete {
                    continue;
                }

                if let Some(value_id) = attr.value {
                    if let Some(AstNode::StringLiteral(lit)) = ctx.node(value_id) {
                        let val = lit.value.as_str().trim();
                        // Autocomplete can have section- prefix and shipping/billing qualifiers
                        let tokens: Vec<&str> = val.split_whitespace().collect();
                        if let Some(last) = tokens.last() {
                            if !VALID_AUTOCOMPLETE.contains(last) && !last.starts_with("section-") {
                                let attr_span = Span::new(attr.span.start, attr.span.end);
                                let fix = FixBuilder::new(
                                    "Remove invalid `autoComplete` attribute",
                                    FixKind::SuggestionFix,
                                )
                                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                                .build();
                                ctx.report(Diagnostic {
                                    rule_name: RULE_NAME.to_owned(),
                                    message: format!("`{val}` is not a valid `autocomplete` value"),
                                    span: Span::new(opening.span.start, opening.span.end),
                                    severity: Severity::Warning,
                                    help: None,
                                    fix,
                                    labels: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(AutocompleteValid)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_invalid_autocomplete() {
        let diags = lint(r#"const el = <input autoComplete="foobar" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_valid_autocomplete() {
        let diags = lint(r#"const el = <input autoComplete="email" />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_autocomplete() {
        let diags = lint(r#"const el = <input type="text" />;"#);
        assert!(diags.is_empty());
    }
}
