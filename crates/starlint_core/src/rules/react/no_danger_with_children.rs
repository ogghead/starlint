//! Rule: `react/no-danger-with-children`
//!
//! Flag elements with both `children` prop/content and `dangerouslySetInnerHTML`.
//! Using both at the same time is invalid because `dangerouslySetInnerHTML`
//! replaces children, so having both is contradictory and causes runtime errors.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeItem;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags elements that use both `dangerouslySetInnerHTML` and children.
#[derive(Debug)]
pub struct NoDangerWithChildren;

impl NativeRule for NoDangerWithChildren {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-danger-with-children".to_owned(),
            description: "Disallow using `dangerouslySetInnerHTML` together with children"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        let opening = &element.opening_element;

        let mut has_danger = false;
        let mut has_children_prop = false;

        for attr in &opening.attributes {
            if let JSXAttributeItem::Attribute(a) = attr {
                if a.is_identifier("dangerouslySetInnerHTML") {
                    has_danger = true;
                } else if a.is_identifier("children") {
                    has_children_prop = true;
                }
            }
        }

        if !has_danger {
            return;
        }

        let has_child_nodes = !element.children.is_empty();

        if has_children_prop || has_child_nodes {
            ctx.report(Diagnostic {
                rule_name: "react/no-danger-with-children".to_owned(),
                message: "Cannot use `dangerouslySetInnerHTML` and `children` at the same time"
                    .to_owned(),
                span: Span::new(element.span.start, element.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDangerWithChildren)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_danger_with_child_nodes() {
        let source = r#"var x = <div dangerouslySetInnerHTML={{ __html: "hi" }}>child</div>;"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "dangerouslySetInnerHTML with child nodes should be flagged"
        );
    }

    #[test]
    fn test_flags_danger_with_children_prop() {
        let source =
            r#"var x = <div dangerouslySetInnerHTML={{ __html: "hi" }} children="child" />;"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "dangerouslySetInnerHTML with children prop should be flagged"
        );
    }

    #[test]
    fn test_allows_danger_alone() {
        let source = r#"var x = <div dangerouslySetInnerHTML={{ __html: "hi" }} />;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "dangerouslySetInnerHTML alone should not be flagged"
        );
    }

    #[test]
    fn test_allows_children_alone() {
        let source = "var x = <div>hello</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "children alone should not be flagged");
    }
}
