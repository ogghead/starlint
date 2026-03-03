//! Rule: `react/self-closing-comp`
//!
//! Components without children should be self-closing. Writing `<Foo></Foo>`
//! when there are no children is unnecessarily verbose.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags JSX elements without children that are not self-closing.
#[derive(Debug)]
pub struct SelfClosingComp;

impl NativeRule for SelfClosingComp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/self-closing-comp".to_owned(),
            description: "Components without children should be self-closing".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        // If there is a closing element, the element is not self-closing.
        // If children is empty but there's a closing tag, it should be self-closing.
        if element.closing_element.is_some() && element.children.is_empty() {
            ctx.report_warning(
                "react/self-closing-comp",
                "Empty components should be self-closing",
                Span::new(element.span.start, element.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SelfClosingComp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_element_with_closing_tag() {
        let source = "var x = <div></div>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "empty element with closing tag should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_component_with_closing_tag() {
        let source = "var x = <MyComponent></MyComponent>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "empty component with closing tag should be flagged"
        );
    }

    #[test]
    fn test_allows_self_closing() {
        let source = "var x = <div />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "self-closing element should not be flagged"
        );
    }

    #[test]
    fn test_allows_element_with_children() {
        let source = "var x = <div>hello</div>;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "element with children should not be flagged"
        );
    }
}
