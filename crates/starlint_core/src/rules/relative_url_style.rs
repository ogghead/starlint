//! Rule: `relative-url-style`
//!
//! Enforce consistent style for relative URL paths. When using `new URL(path, base)`,
//! relative paths should start with `./` for clarity. Flags `new URL('foo', base)`
//! that should be `new URL('./foo', base)`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Prefixes that indicate the URL path is already explicit (not a bare relative path).
const EXPLICIT_PREFIXES: &[&str] = &["./", "../", "http://", "https://", "/", "#"];

/// Flags bare relative paths in `new URL()` calls.
#[derive(Debug)]
pub struct RelativeUrlStyle;

impl NativeRule for RelativeUrlStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "relative-url-style".to_owned(),
            description: "Enforce `./` prefix for relative URL paths".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        // Check if callee is `URL`
        let Expression::Identifier(callee) = &new_expr.callee else {
            return;
        };

        if callee.name.as_str() != "URL" {
            return;
        }

        // Must have at least two arguments (path and base URL)
        if new_expr.arguments.len() < 2 {
            return;
        }

        // Get the first argument (the URL path)
        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        // Must be a string literal
        let Argument::StringLiteral(path_lit) = first_arg else {
            return;
        };

        let path = path_lit.value.as_str();

        // Check if the path is a bare relative path (no explicit prefix)
        let is_explicit = EXPLICIT_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix));

        if !is_explicit && !path.is_empty() {
            // Fix: replace string content with `./` prefix
            // String literal span includes quotes, so skip them
            let inner_start = path_lit.span.start.saturating_add(1);
            let inner_end = path_lit.span.end.saturating_sub(1);
            let path_owned = path.to_owned();
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: "Add `./` prefix".to_owned(),
                edits: vec![Edit {
                    span: Span::new(inner_start, inner_end),
                    replacement: format!("./{path_owned}"),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "relative-url-style".to_owned(),
                message: format!(
                    "Relative URL '{path_owned}' should start with `./` — use './{path_owned}' instead"
                ),
                span: Span::new(new_expr.span.start, new_expr.span.end),
                severity: Severity::Warning,
                help: Some(format!("Use './{path_owned}' instead")),
                fix,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RelativeUrlStyle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bare_relative_path() {
        let diags = lint("new URL('foo', base);");
        assert_eq!(
            diags.len(),
            1,
            "bare relative path in URL should be flagged"
        );
    }

    #[test]
    fn test_allows_dot_slash_prefix() {
        let diags = lint("new URL('./foo', base);");
        assert!(
            diags.is_empty(),
            "path with ./ prefix should not be flagged"
        );
    }

    #[test]
    fn test_allows_dot_dot_slash_prefix() {
        let diags = lint("new URL('../foo', base);");
        assert!(
            diags.is_empty(),
            "path with ../ prefix should not be flagged"
        );
    }

    #[test]
    fn test_allows_absolute_url() {
        let diags = lint("new URL('https://example.com');");
        assert!(diags.is_empty(), "absolute URL should not be flagged");
    }

    #[test]
    fn test_allows_root_relative_path() {
        let diags = lint("new URL('/foo', base);");
        assert!(diags.is_empty(), "root-relative path should not be flagged");
    }

    #[test]
    fn test_allows_single_arg_url() {
        let diags = lint("new URL('foo');");
        assert!(
            diags.is_empty(),
            "single-arg URL (no base) should not be flagged"
        );
    }

    #[test]
    fn test_allows_hash_fragment() {
        let diags = lint("new URL('#section', base);");
        assert!(diags.is_empty(), "hash fragment should not be flagged");
    }
}
