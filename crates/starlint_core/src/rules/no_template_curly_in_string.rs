//! Rule: `no-template-curly-in-string`
//!
//! Disallow template literal placeholder syntax in regular strings.
//! Writing `"Hello ${name}"` instead of `` `Hello ${name}` `` is a common
//! mistake — the `${...}` is treated as literal text, not as interpolation.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags regular string literals that contain `${...}` template syntax.
#[derive(Debug)]
pub struct NoTemplateCurlyInString;

impl NativeRule for NoTemplateCurlyInString {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-template-curly-in-string".to_owned(),
            description: "Disallow template literal placeholder syntax in regular strings"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StringLiteral(lit) = kind else {
            return;
        };

        let value = lit.value.as_str();
        if contains_template_placeholder(value) {
            ctx.report_error(
                "no-template-curly-in-string",
                "Unexpected template string expression in a regular string",
                Span::new(lit.span.start, lit.span.end),
            );
        }
    }
}

/// Check if a string contains what looks like a template placeholder `${...}`.
fn contains_template_placeholder(s: &str) -> bool {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes.get(i).copied() == Some(b'$') {
            let next = i.checked_add(1).unwrap_or(len);
            if bytes.get(next).copied() == Some(b'{') {
                // Look for the closing brace
                let search_start = next.checked_add(1).unwrap_or(len);
                let mut j = search_start;
                while j < len {
                    if bytes.get(j).copied() == Some(b'}') {
                        return true;
                    }
                    j = j.checked_add(1).unwrap_or(len);
                }
            }
        }
        i = i.checked_add(1).unwrap_or(len);
    }
    false
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTemplateCurlyInString)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_template_in_double_quotes() {
        let diags = lint(r#"var x = "Hello ${name}";"#);
        assert_eq!(
            diags.len(),
            1,
            "template placeholder in string should be flagged"
        );
    }

    #[test]
    fn test_flags_template_in_single_quotes() {
        let diags = lint("var x = 'Hello ${name}';");
        assert_eq!(
            diags.len(),
            1,
            "template placeholder in single-quoted string should be flagged"
        );
    }

    #[test]
    fn test_allows_template_literal() {
        let diags = lint("var x = `Hello ${name}`;");
        assert!(diags.is_empty(), "template literal should not be flagged");
    }

    #[test]
    fn test_allows_dollar_without_brace() {
        let diags = lint(r#"var x = "price is $5";"#);
        assert!(diags.is_empty(), "$5 without braces should not be flagged");
    }

    #[test]
    fn test_allows_plain_string() {
        let diags = lint(r#"var x = "hello world";"#);
        assert!(diags.is_empty(), "plain string should not be flagged");
    }

    #[test]
    fn test_flags_expression_template() {
        let diags = lint(r#"var x = "result: ${a + b}";"#);
        assert_eq!(
            diags.len(),
            1,
            "template expression in string should be flagged"
        );
    }

    #[test]
    fn test_allows_unclosed_placeholder() {
        let diags = lint(r#"var x = "price: ${unclosed";"#);
        assert!(
            diags.is_empty(),
            "unclosed placeholder should not be flagged"
        );
    }
}
