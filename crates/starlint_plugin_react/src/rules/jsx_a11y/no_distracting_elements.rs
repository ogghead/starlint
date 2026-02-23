//! Rule: `jsx-a11y/no-distracting-elements`
//!
//! Forbid `<marquee>` and `<blink>` elements.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/no-distracting-elements";

/// Distracting element names.
const DISTRACTING_ELEMENTS: &[&str] = &["marquee", "blink"];

#[derive(Debug)]
pub struct NoDistractingElements;

impl LintRule for NoDistractingElements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<marquee>` and `<blink>` elements".to_owned(),
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

        let element_name = opening.name.as_str();

        if DISTRACTING_ELEMENTS.contains(&element_name) {
            let fix = build_replace_fix(ctx.source_text(), opening, element_name);
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`<{element_name}>` is distracting and must not be used"),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `<span>`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Build a fix that replaces both opening and closing tag names with `span`.
#[allow(clippy::as_conversions)] // u32↔usize lossless on 32/64-bit
fn build_replace_fix(
    source: &str,
    opening: &starlint_ast::node::JSXOpeningElementNode,
    element_name: &str,
) -> Option<Fix> {
    // The element name starts after `<` in the opening tag
    // opening.span.start points to `<`, so name starts at +1
    let name_start = opening.span.start.saturating_add(1);
    let name_end = name_start.saturating_add(u32::try_from(element_name.len()).unwrap_or(0));

    let mut edits = vec![Edit {
        span: Span::new(name_start, name_end),
        replacement: "span".to_owned(),
    }];

    // Find the closing tag name in the source after the opening element.
    let opening_end = opening.span.end as usize;
    let close_tag = format!("</{element_name}>");
    if let Some(close_offset) = source.get(opening_end..)?.find(&close_tag) {
        let close_name_start = opening_end.saturating_add(close_offset).saturating_add(2); // skip "</"
        let close_name_end = close_name_start.saturating_add(element_name.len());
        edits.push(Edit {
            span: Span::new(
                u32::try_from(close_name_start).ok()?,
                u32::try_from(close_name_end).ok()?,
            ),
            replacement: "span".to_owned(),
        });
    }

    Some(Fix {
        kind: FixKind::SuggestionFix,
        message: format!("Replace `<{element_name}>` with `<span>`"),
        edits,
        is_snippet: false,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDistractingElements)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_marquee() {
        let diags = lint(r"const el = <marquee>scrolling text</marquee>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_blink() {
        let diags = lint(r"const el = <blink>blinking text</blink>;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_normal_elements() {
        let diags = lint(r"const el = <div>content</div>;");
        assert!(diags.is_empty());
    }
}
