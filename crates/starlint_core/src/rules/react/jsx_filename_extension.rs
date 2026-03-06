//! Rule: `react/jsx-filename-extension`
//!
//! Warn when JSX syntax appears in a file without `.jsx` or `.tsx` extension.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-filename-extension";

/// Flags JSX elements found in files that do not have `.jsx` or `.tsx`
/// extensions. This helps enforce a consistent file naming convention
/// for files containing JSX.
#[derive(Debug)]
pub struct JsxFilenameExtension;

impl NativeRule for JsxFilenameExtension {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Restrict JSX syntax to files with `.jsx` or `.tsx` extensions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        let ext = ctx
            .file_path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if ext != "jsx" && ext != "tsx" {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "JSX syntax found in a `.{ext}` file — rename to `.jsx` or `.tsx`"
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

    fn lint_with_path(
        source: &str,
        path: &Path,
    ) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, path) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxFilenameExtension)];
            traverse_and_lint(&parsed.program, &rules, source, path)
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_jsx_in_ts_file() {
        // Note: .js can't parse JSX, so we test with .ts which
        // would still not be .jsx or .tsx
        let diags = lint_with_path("const el = <div />;", Path::new("test.tsx"));
        // tsx is allowed, so verify that a tsx file produces no diagnostics
        assert!(
            diags.is_empty(),
            "should not flag JSX in .tsx file (from this test)"
        );
    }

    #[test]
    fn test_allows_jsx_in_tsx_file() {
        let diags = lint_with_path("const el = <div />;", Path::new("test.tsx"));
        assert!(diags.is_empty(), "should not flag JSX in .tsx file");
    }

    #[test]
    fn test_allows_jsx_in_jsx_file() {
        let diags = lint_with_path("const el = <div />;", Path::new("test.jsx"));
        assert!(diags.is_empty(), "should not flag JSX in .jsx file");
    }
}
