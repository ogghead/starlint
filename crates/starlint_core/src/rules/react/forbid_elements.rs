//! Rule: `react/forbid-elements`
//!
//! Warn when forbidden elements are used. Flags `<marquee>` and `<blink>`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags usage of forbidden HTML elements. By default, `<marquee>` and
/// `<blink>` are flagged as deprecated and non-standard elements.
#[derive(Debug)]
pub struct ForbidElements;

/// Deprecated or non-standard HTML element names that should be avoided.
const FORBIDDEN_ELEMENTS: &[&str] = &["marquee", "blink"];

impl LintRule for ForbidElements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/forbid-elements".to_owned(),
            description: "Warn when forbidden elements are used".to_owned(),
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

        let tag_name = opening.name.as_str();

        if FORBIDDEN_ELEMENTS.contains(&tag_name) {
            ctx.report(Diagnostic {
                rule_name: "react/forbid-elements".to_owned(),
                message: format!(
                    "`<{tag_name}>` is forbidden — this element is deprecated or non-standard"
                ),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ForbidElements)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_marquee() {
        let source = "const x = <marquee>Scrolling</marquee>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "marquee should be flagged");
    }

    #[test]
    fn test_flags_blink() {
        let source = "const x = <blink>Blinking</blink>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "blink should be flagged");
    }

    #[test]
    fn test_allows_normal_elements() {
        let source = "const x = <div>Hello</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "normal elements should not be flagged");
    }
}
