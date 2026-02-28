//! Rule: `no-script-url`
//!
//! Disallow `javascript:` URLs. These are a form of `eval()` and pose
//! security risks (XSS).

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags string literals that contain `javascript:` URLs.
#[derive(Debug)]
pub struct NoScriptUrl;

impl NativeRule for NoScriptUrl {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-script-url".to_owned(),
            description: "Disallow `javascript:` URLs".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StringLiteral(lit) = kind else {
            return;
        };

        if lit.value.to_lowercase().starts_with("javascript:") {
            ctx.report_error(
                "no-script-url",
                "Script URL is a form of `eval()` and is a security risk",
                Span::new(lit.span.start, lit.span.end),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoScriptUrl)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_javascript_url() {
        let diags = lint("var url = 'javascript:void(0)';");
        assert_eq!(diags.len(), 1, "javascript: URL should be flagged");
    }

    #[test]
    fn test_flags_javascript_url_mixed_case() {
        let diags = lint("var url = 'JavaScript:alert(1)';");
        assert_eq!(
            diags.len(),
            1,
            "mixed-case javascript: URL should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_url() {
        let diags = lint("var url = 'https://example.com';");
        assert!(diags.is_empty(), "normal URL should not be flagged");
    }

    #[test]
    fn test_allows_non_url_string() {
        let diags = lint("var msg = 'hello world';");
        assert!(diags.is_empty(), "normal string should not be flagged");
    }
}
