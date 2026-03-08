//! Rule: `relative-url-style`
//!
//! Enforce consistent style for relative URL paths. When using `new URL(path, base)`,
//! relative paths should start with `./` for clarity. Flags `new URL('foo', base)`
//! that should be `new URL('./foo', base)`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Prefixes that indicate the URL path is already explicit (not a bare relative path).
const EXPLICIT_PREFIXES: &[&str] = &["./", "../", "http://", "https://", "/", "#"];

/// Flags bare relative paths in `new URL()` calls.
#[derive(Debug)]
pub struct RelativeUrlStyle;

impl LintRule for RelativeUrlStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "relative-url-style".to_owned(),
            description: "Enforce `./` prefix for relative URL paths".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::NewExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::NewExpression(new_expr) = node else {
            return;
        };

        // Check if callee is `URL`
        let is_url = matches!(ctx.node(new_expr.callee), Some(AstNode::IdentifierReference(callee)) if callee.name.as_str() == "URL");
        if !is_url {
            return;
        }

        // Must have at least two arguments (path and base URL)
        if new_expr.arguments.len() < 2 {
            return;
        }

        // Get the first argument (the URL path)
        let Some(&first_arg_id) = new_expr.arguments.first() else {
            return;
        };

        // Must be a string literal — extract data before calling ctx.report()
        let (path_value, lit_span) = {
            let Some(AstNode::StringLiteral(path_lit)) = ctx.node(first_arg_id) else {
                return;
            };
            (path_lit.value.clone(), path_lit.span)
        };

        // Check if the path is a bare relative path (no explicit prefix)
        let is_explicit = EXPLICIT_PREFIXES
            .iter()
            .any(|prefix| path_value.starts_with(prefix));

        if !is_explicit && !path_value.is_empty() {
            // Fix: replace string content with `./` prefix
            // String literal span includes quotes, so skip them
            let inner_start = lit_span.start.saturating_add(1);
            let inner_end = lit_span.end.saturating_sub(1);
            let path_owned = path_value;
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

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RelativeUrlStyle)];
        lint_source(source, "test.js", &rules)
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
