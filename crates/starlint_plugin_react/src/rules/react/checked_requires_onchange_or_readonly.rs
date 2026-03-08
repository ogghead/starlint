//! Rule: `react/checked-requires-onchange-or-readonly`
//!
//! Warn when `checked` prop is used without `onChange` or `readOnly`.

#![allow(clippy::match_same_arms)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags JSX elements that have a `checked` prop but lack both `onChange`
/// and `readOnly` props. This causes the component to be a read-only
/// controlled input without proper handling.
#[derive(Debug)]
pub struct CheckedRequiresOnchangeOrReadonly;

/// Extract the plain attribute name from a JSX attribute `NodeId`, if any.
fn attr_name<'a>(item_id: NodeId, ctx: &'a LintContext<'_>) -> Option<&'a str> {
    match ctx.node(item_id)? {
        AstNode::JSXAttribute(a) => Some(a.name.as_str()),
        AstNode::JSXSpreadAttribute(_) => None,
        _ => None,
    }
}

impl LintRule for CheckedRequiresOnchangeOrReadonly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/checked-requires-onchange-or-readonly".to_owned(),
            description: "Require onChange or readOnly when using checked prop".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    #[allow(clippy::match_same_arms)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        let mut has_checked = false;
        let mut has_on_change = false;
        let mut has_read_only = false;

        for &attr_id in &*opening.attributes {
            if let Some(name) = attr_name(attr_id, ctx) {
                match name {
                    "checked" => has_checked = true,
                    "onChange" => has_on_change = true,
                    "readOnly" => has_read_only = true,
                    _ => {}
                }
            }
        }

        if has_checked && !has_on_change && !has_read_only {
            let insert_pos = fix_utils::jsx_attr_insert_offset(
                ctx.source_text(),
                Span::new(opening.span.start, opening.span.end),
            );
            let fix = FixBuilder::new("Add `readOnly`", FixKind::SuggestionFix)
                .insert_at(insert_pos, " readOnly")
                .build();
            ctx.report(Diagnostic {
                rule_name: "react/checked-requires-onchange-or-readonly".to_owned(),
                message: "`checked` prop requires `onChange` or `readOnly` to avoid a read-only controlled input".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: Some("Add `readOnly` or `onChange`".to_owned()),
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(CheckedRequiresOnchangeOrReadonly)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_checked_without_onchange_or_readonly() {
        let source = "const x = <input checked />;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "checked without onChange or readOnly should be flagged"
        );
    }

    #[test]
    fn test_allows_checked_with_onchange() {
        let source = "const x = <input checked onChange={handleChange} />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "checked with onChange should not be flagged"
        );
    }

    #[test]
    fn test_allows_checked_with_readonly() {
        let source = "const x = <input checked readOnly />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "checked with readOnly should not be flagged"
        );
    }
}
