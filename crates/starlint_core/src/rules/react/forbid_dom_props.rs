//! Rule: `react/forbid-dom-props`
//!
//! Warn when certain DOM props are used. Default: flags `id` prop on DOM elements.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags use of forbidden DOM props. By default, flags the `id` prop on
/// lowercase (DOM) elements as a hint that IDs are often an anti-pattern
/// in component-based architectures.
#[derive(Debug)]
pub struct ForbidDomProps;

/// Default set of forbidden DOM props.
const FORBIDDEN_PROPS: &[&str] = &["id"];

impl NativeRule for ForbidDomProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/forbid-dom-props".to_owned(),
            description: "Warn when certain DOM props are used".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXAttribute])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        // Get the attribute name
        let attr_name = match &attr.name {
            JSXAttributeName::Identifier(id) => id.name.as_str(),
            JSXAttributeName::NamespacedName(_) => return,
        };

        // Only flag forbidden props
        if !FORBIDDEN_PROPS.contains(&attr_name) {
            return;
        }

        // Check if this attribute is on a DOM element by scanning the source
        // text backwards from the attribute to find the opening tag name.
        // We use the heuristic that the JSXAttribute's parent is a JSXOpeningElement
        // whose name starts with a lowercase letter.
        let source = ctx.source_text();
        let attr_start = usize::try_from(attr.span.start).unwrap_or(0);
        if attr_start == 0 {
            return;
        }

        // Scan backward to find `<tagname` pattern
        let before = &source[..attr_start];
        // Find the last `<` before this attribute
        if let Some(lt_pos) = before.rfind('<') {
            let after_lt = &source[lt_pos.saturating_add(1)..attr_start];
            // Extract the tag name (first word after `<`)
            let tag_name = after_lt.split_whitespace().next().unwrap_or("");
            if !tag_name.is_empty()
                && tag_name
                    .as_bytes()
                    .first()
                    .is_some_and(|&b| b.is_ascii_lowercase())
            {
                let attr_span = Span::new(attr.span.start, attr.span.end);
                let fix =
                    FixBuilder::new(format!("Remove `{attr_name}` prop"), FixKind::SuggestionFix)
                        .edit(fix_utils::remove_jsx_attr(source, attr_span))
                        .build();
                ctx.report(Diagnostic {
                    rule_name: "react/forbid-dom-props".to_owned(),
                    message: format!("Prop `{attr_name}` is forbidden on DOM elements"),
                    span: attr_span,
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ForbidDomProps)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_id_on_dom_element() {
        let source = r#"const x = <div id="main" />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "id prop on DOM element should be flagged");
    }

    #[test]
    fn test_allows_id_on_component() {
        let source = r#"const x = <MyComponent id="main" />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "id prop on React component should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_props_on_dom_element() {
        let source = r#"const x = <div className="foo" />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-forbidden prop on DOM element should not be flagged"
        );
    }
}
