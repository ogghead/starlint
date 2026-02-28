//! Rule: `react/no-danger`
//!
//! Flag `dangerouslySetInnerHTML` prop usage. Using `dangerouslySetInnerHTML`
//! bypasses React's DOM sanitization and exposes the application to XSS
//! attacks if the HTML content is not properly sanitized.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of the `dangerouslySetInnerHTML` prop.
#[derive(Debug)]
pub struct NoDanger;

impl NativeRule for NoDanger {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-danger".to_owned(),
            description: "Disallow usage of `dangerouslySetInnerHTML`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        if attr.is_identifier("dangerouslySetInnerHTML") {
            ctx.report_warning(
                "react/no-danger",
                "Avoid using `dangerouslySetInnerHTML` -- it exposes your app to XSS attacks",
                Span::new(attr.span.start, attr.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDanger)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_dangerously_set_inner_html() {
        let source = r#"var x = <div dangerouslySetInnerHTML={{ __html: "<b>bold</b>" }} />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "dangerouslySetInnerHTML should be flagged");
    }

    #[test]
    fn test_allows_normal_props() {
        let source = r#"var x = <div className="foo" />;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "normal props should not be flagged");
    }

    #[test]
    fn test_flags_on_custom_component() {
        let source = r#"var x = <MyComponent dangerouslySetInnerHTML={{ __html: "hi" }} />;"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "dangerouslySetInnerHTML on custom component should be flagged"
        );
    }
}
