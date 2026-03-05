//! Rule: `react/jsx-max-depth`
//!
//! Warn when JSX nesting exceeds a reasonable depth (default 10).

use oxc_ast::AstKind;
use oxc_ast::ast::JSXChild;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-max-depth";

/// Default maximum JSX nesting depth.
const DEFAULT_MAX_DEPTH: usize = 10;

/// Flags JSX elements that are nested deeper than the configured maximum depth.
/// Deep nesting is a code smell indicating the component should be broken up.
#[derive(Debug)]
pub struct JsxMaxDepth;

/// Recursively compute the maximum JSX nesting depth of an element's children.
fn jsx_depth(children: &[JSXChild<'_>]) -> usize {
    let mut max = 0;
    for child in children {
        let child_depth = match child {
            JSXChild::Element(el) => jsx_depth(&el.children).saturating_add(1),
            JSXChild::Fragment(frag) => jsx_depth(&frag.children),
            _ => 0,
        };
        if child_depth > max {
            max = child_depth;
        }
    }
    max
}

impl NativeRule for JsxMaxDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce a maximum JSX nesting depth".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        let depth = jsx_depth(&element.children).saturating_add(1);
        if depth > DEFAULT_MAX_DEPTH {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "JSX nesting depth of {depth} exceeds maximum of {DEFAULT_MAX_DEPTH}. Consider extracting sub-components"
                ),
                span: Span::new(element.span.start, element.span.end),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxMaxDepth)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_shallow_nesting() {
        let diags = lint("const el = <div><span><a>hi</a></span></div>;");
        assert!(
            diags.is_empty(),
            "should not flag shallow nesting (depth 3)"
        );
    }

    #[test]
    fn test_flags_deep_nesting() {
        // Build nesting of depth 11
        let mut source = String::from("const el = ");
        for _ in 0..11 {
            source.push_str("<div>");
        }
        source.push_str("hi");
        for _ in 0..11 {
            source.push_str("</div>");
        }
        source.push(';');
        let diags = lint(&source);
        assert!(
            !diags.is_empty(),
            "should flag nesting exceeding max depth of 10"
        );
    }

    #[test]
    fn test_allows_exactly_max_depth() {
        // Build nesting of exactly depth 10
        let mut source = String::from("const el = ");
        for _ in 0..10 {
            source.push_str("<div>");
        }
        source.push_str("hi");
        for _ in 0..10 {
            source.push_str("</div>");
        }
        source.push(';');
        let diags = lint(&source);
        assert!(
            diags.is_empty(),
            "should not flag nesting at exactly max depth"
        );
    }
}
