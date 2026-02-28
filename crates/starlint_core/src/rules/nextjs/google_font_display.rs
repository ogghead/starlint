//! Rule: `nextjs/google-font-display`
//!
//! Enforce `display` parameter in Google Fonts URLs to avoid invisible text
//! during font loading (FOIT).

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/google-font-display";

/// Flags Google Fonts URLs that are missing the `display` query parameter.
#[derive(Debug)]
pub struct GoogleFontDisplay;

impl NativeRule for GoogleFontDisplay {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `display` parameter in Google Fonts URLs".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StringLiteral(lit) = kind else {
            return;
        };

        let value = lit.value.as_str();

        if !value.contains("fonts.googleapis.com") {
            return;
        }

        // Check for the `display=` query parameter
        let has_display = value.contains("&display=") || value.contains("?display=");

        if !has_display {
            ctx.report_warning(
                RULE_NAME,
                "Google Fonts URL is missing the `display` parameter. Add `&display=swap` to avoid invisible text during loading",
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GoogleFontDisplay)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_missing_display() {
        let diags = lint(r#"const url = "https://fonts.googleapis.com/css?family=Roboto";"#);
        assert_eq!(diags.len(), 1, "missing display param should be flagged");
    }

    #[test]
    fn test_allows_with_display() {
        let diags =
            lint(r#"const url = "https://fonts.googleapis.com/css?family=Roboto&display=swap";"#);
        assert!(diags.is_empty(), "URL with display param should pass");
    }

    #[test]
    fn test_ignores_non_google_fonts() {
        let diags = lint(r#"const url = "https://example.com/fonts";"#);
        assert!(
            diags.is_empty(),
            "non-Google Fonts URL should not be flagged"
        );
    }
}
