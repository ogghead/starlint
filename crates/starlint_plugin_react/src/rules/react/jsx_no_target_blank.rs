//! Rule: `react/jsx-no-target-blank`
//!
//! Warn when `<a target="_blank">` is used without `rel="noreferrer"`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::jsx_utils::get_jsx_attr_string_value;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-target-blank";

/// Flags `<a target="_blank">` elements that are missing `rel="noreferrer"`.
#[derive(Debug)]
pub struct JsxNoTargetBlank;

/// Get the span of a named attribute.
fn get_attr_span(attributes: &[NodeId], attr_name: &str, ctx: &LintContext<'_>) -> Option<Span> {
    for &attr_id in attributes {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) {
            if attr.name == attr_name {
                return Some(Span::new(attr.span.start, attr.span.end));
            }
        }
    }
    None
}

impl LintRule for JsxNoTargetBlank {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when `<a target=\"_blank\">` is missing `rel=\"noreferrer\"`"
                .to_owned(),
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

        // Only check `<a>` elements
        if opening.name != "a" {
            return;
        }

        let opening_span = opening.span;
        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Check for target="_blank"
        let has_target_blank =
            get_jsx_attr_string_value(&attrs, "target", ctx).as_deref() == Some("_blank");

        if !has_target_blank {
            return;
        }

        // Check for rel containing "noreferrer"
        let has_noreferrer = get_jsx_attr_string_value(&attrs, "rel", ctx)
            .is_some_and(|val| val.split_whitespace().any(|part| part == "noreferrer"));

        if !has_noreferrer {
            let span = Span::new(opening_span.start, opening_span.end);

            // Find existing rel attribute to determine fix strategy
            let rel_span = get_attr_span(&attrs, "rel", ctx);
            let existing_rel_val = get_jsx_attr_string_value(&attrs, "rel", ctx);

            let fix = if let Some(rel_s) = rel_span {
                // Existing rel attribute: replace its value to include "noreferrer"
                let existing_value = existing_rel_val.as_deref().unwrap_or("");
                let new_value = if existing_value.is_empty() {
                    "noreferrer".to_owned()
                } else {
                    format!("{existing_value} noreferrer")
                };
                Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `noreferrer` to the `rel` attribute".to_owned(),
                    edits: vec![Edit {
                        span: rel_s,
                        replacement: format!("rel=\"{new_value}\""),
                    }],
                    is_snippet: false,
                })
            } else {
                // No rel attribute: insert before the closing `>` or `/>` of the opening tag
                let open_end = opening_span.end;
                let source = ctx.source_text();
                let end_idx = usize::try_from(open_end).unwrap_or(0);
                let before_end = source.get(end_idx.saturating_sub(2)..end_idx).unwrap_or("");
                let insert_pos = if before_end.ends_with("/>") {
                    open_end.saturating_sub(2)
                } else {
                    open_end.saturating_sub(1)
                };
                let insert_span = Span::new(insert_pos, insert_pos);
                Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `rel=\"noreferrer\"`".to_owned(),
                    edits: vec![Edit {
                        span: insert_span,
                        replacement: " rel=\"noreferrer\"".to_owned(),
                    }],
                    is_snippet: false,
                })
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Using `target=\"_blank\"` without `rel=\"noreferrer\"` is a security risk"
                        .to_owned(),
                span,
                severity: Severity::Warning,
                help: Some("Add `rel=\"noreferrer\"` to the element".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(JsxNoTargetBlank);

    #[test]
    fn test_flags_target_blank_without_rel() {
        let diags = lint(r#"const el = <a href="https://example.com" target="_blank">link</a>;"#);
        assert_eq!(diags.len(), 1, "should flag missing rel=noreferrer");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_with_noreferrer() {
        let diags = lint(
            r#"const el = <a href="https://example.com" target="_blank" rel="noreferrer">link</a>;"#,
        );
        assert!(diags.is_empty(), "should not flag when noreferrer present");
    }

    #[test]
    fn test_allows_no_target_blank() {
        let diags = lint(r#"const el = <a href="https://example.com">link</a>;"#);
        assert!(
            diags.is_empty(),
            "should not flag anchor without target=_blank"
        );
    }
}
