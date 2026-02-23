//! Rule: `no-div-regex`
//!
//! Disallow regular expressions that look like division. A regex like
//! `/=foo/` can be confused with a division assignment and should be
//! written as `/[=]foo/` or `new RegExp("=foo")`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags regex literals that start with `=`.
#[derive(Debug)]
pub struct NoDivRegex;

impl LintRule for NoDivRegex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-div-regex".to_owned(),
            description: "Disallow regular expressions that look like division".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::RegExpLiteral])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::RegExpLiteral(regex) = node else {
            return;
        };

        if regex.pattern.starts_with('=') {
            // Fix: escape the leading = by wrapping in char class [=]
            let source = ctx.source_text();
            let start = regex.span.start as usize;
            let end = regex.span.end as usize;
            let raw = source.get(start..end).unwrap_or("");
            // raw is "/=pattern/flags" — insert [=] after first /
            let fix = raw.get(2..).map(|rest| Fix {
                kind: FixKind::SafeFix,
                message: "Escape leading `=` in regex".to_owned(),
                edits: vec![Edit {
                    span: Span::new(regex.span.start, regex.span.end),
                    replacement: format!("/[=]{rest}"),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "no-div-regex".to_owned(),
                message: "Ambiguous regex: looks like it could be a division operator".to_owned(),
                span: Span::new(regex.span.start, regex.span.end),
                severity: Severity::Warning,
                help: Some("Escape leading `=` with `[=]`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDivRegex)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_div_like_regex() {
        let diags = lint("var r = /=foo/;");
        assert_eq!(diags.len(), 1, "/=foo/ should be flagged");
    }

    #[test]
    fn test_allows_normal_regex() {
        let diags = lint("var r = /foo/;");
        assert!(diags.is_empty(), "normal regex should not be flagged");
    }

    #[test]
    fn test_allows_char_class_regex() {
        let diags = lint("var r = /[=]foo/;");
        assert!(
            diags.is_empty(),
            "regex with = in char class should not be flagged"
        );
    }
}
