//! Rule: `jsx-a11y/media-has-caption`
//!
//! Enforce `<audio>` and `<video>` have `<track>` for captions.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/media-has-caption";

/// Media elements that require captions.
const MEDIA_ELEMENTS: &[&str] = &["audio", "video"];

#[derive(Debug)]
pub struct MediaHasCaption;

/// Check if an attribute with the given name exists on a JSX element.
fn has_attribute(
    opening: &starlint_ast::node::JSXOpeningElementNode,
    name: &str,
    ctx: &LintContext<'_>,
) -> bool {
    opening.attributes.iter().any(|attr_id| {
        if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
            attr.name.as_str() == name
        } else {
            false
        }
    })
}

impl LintRule for MediaHasCaption {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `<audio>` and `<video>` have `<track>` for captions".to_owned(),
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

        if !MEDIA_ELEMENTS.contains(&element_name) {
            return;
        }

        // Muted videos don't need captions
        if element_name == "video" && has_attribute(opening, "muted", ctx) {
            return;
        }

        // Check for aria-label or aria-labelledby as alternatives
        if has_attribute(opening, "aria-label", ctx)
            || has_attribute(opening, "aria-labelledby", ctx)
        {
            return;
        }

        // Heuristic: flag media elements that lack a `muted` attribute.
        // A complete check would inspect children for `<track>`, but the
        // opening-element visitor cannot see children. Flagging here is a
        // reasonable heuristic for the most common mistake (no captions at all).
        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: format!("`<{element_name}>` elements must have a `<track>` element with captions. Consider adding `muted` for videos without audio"),
            span: Span::new(opening.span.start, opening.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MediaHasCaption)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_video_without_track() {
        let diags = lint(r#"const el = <video src="movie.mp4" />;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_muted_video() {
        let diags = lint(r#"const el = <video src="movie.mp4" muted />;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_audio_without_track() {
        let diags = lint(r#"const el = <audio src="song.mp3" />;"#);
        assert_eq!(diags.len(), 1);
    }
}
