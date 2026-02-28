//! Rule: `react/style-prop-object`
//!
//! The `style` prop should be an object. Passing a string as the `style` prop
//! in JSX is a common mistake when migrating from HTML -- React requires style
//! to be a JavaScript object.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeValue;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `style` props with string literal values.
#[derive(Debug)]
pub struct StylePropObject;

impl NativeRule for StylePropObject {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/style-prop-object".to_owned(),
            description: "The `style` prop should be an object, not a string".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        if !attr.is_identifier("style") {
            return;
        }

        // Check if the value is a string literal
        if let Some(JSXAttributeValue::StringLiteral(_)) = &attr.value {
            ctx.report_error(
                "react/style-prop-object",
                "The `style` prop expects an object, not a string",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(StylePropObject)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_style_string() {
        let source = r#"var x = <div style="color: red" />;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "string style prop should be flagged");
    }

    #[test]
    fn test_allows_style_object() {
        let source = "var x = <div style={{ color: 'red' }} />;";
        let diags = lint(source);
        assert!(diags.is_empty(), "object style prop should not be flagged");
    }

    #[test]
    fn test_allows_other_string_props() {
        let source = r#"var x = <div className="foo" />;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "other string props should not be flagged");
    }
}
