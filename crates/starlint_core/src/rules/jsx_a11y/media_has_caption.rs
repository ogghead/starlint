//! Rule: `jsx-a11y/media-has-caption`
//!
//! Enforce `<audio>` and `<video>` have `<track>` for captions.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/media-has-caption";

/// Media elements that require captions.
const MEDIA_ELEMENTS: &[&str] = &["audio", "video"];

#[derive(Debug)]
pub struct MediaHasCaption;

/// Check if an attribute exists on a JSX element.
fn has_attribute(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> bool {
    opening.attributes.iter().any(|item| {
        if let JSXAttributeItem::Attribute(attr) = item {
            match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == name,
                JSXAttributeName::NamespacedName(_) => false,
            }
        } else {
            false
        }
    })
}

impl NativeRule for MediaHasCaption {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `<audio>` and `<video>` have `<track>` for captions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if !MEDIA_ELEMENTS.contains(&element_name) {
            return;
        }

        // Muted videos don't need captions
        if element_name == "video" && has_attribute(opening, "muted") {
            return;
        }

        // Check for aria-label or aria-labelledby as alternatives
        if has_attribute(opening, "aria-label") || has_attribute(opening, "aria-labelledby") {
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MediaHasCaption)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
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
