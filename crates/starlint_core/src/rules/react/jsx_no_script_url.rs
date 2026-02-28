//! Rule: `react/jsx-no-script-url`
//!
//! Error when JSX attributes contain `javascript:` URLs.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeValue, JSXExpression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-script-url";

/// Flags JSX attributes that contain `javascript:` URLs, which are a security
/// risk (XSS vector).
#[derive(Debug)]
pub struct JsxNoScriptUrl;

/// Check if a string value starts with `javascript:` (case-insensitive).
fn is_script_url(value: &str) -> bool {
    value.trim().to_ascii_lowercase().starts_with("javascript:")
}

impl NativeRule for JsxNoScriptUrl {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `javascript:` URLs in JSX attributes".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        let Some(value) = &attr.value else {
            return;
        };

        let has_script_url = match value {
            JSXAttributeValue::StringLiteral(lit) => is_script_url(lit.value.as_str()),
            JSXAttributeValue::ExpressionContainer(container) => {
                if let JSXExpression::StringLiteral(lit) = &container.expression {
                    is_script_url(lit.value.as_str())
                } else {
                    false
                }
            }
            _ => false,
        };

        if has_script_url {
            ctx.report_error(
                RULE_NAME,
                "Disallow `javascript:` URLs — they are a security risk",
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoScriptUrl)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_javascript_url_string() {
        let diags = lint(r#"const el = <a href="javascript:alert('xss')">link</a>;"#);
        assert_eq!(diags.len(), 1, "should flag javascript: URL");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_javascript_url_expression() {
        let diags = lint(r#"const el = <a href={"javascript:void(0)"}>link</a>;"#);
        assert_eq!(
            diags.len(),
            1,
            "should flag javascript: URL in expression container"
        );
    }

    #[test]
    fn test_allows_normal_url() {
        let diags = lint(r#"const el = <a href="https://example.com">link</a>;"#);
        assert!(diags.is_empty(), "should not flag normal URLs");
    }
}
