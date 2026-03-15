//! Rule: `react/no-unescaped-entities`
//!
//! Warn about unescaped `>`, `"`, `'`, `}` characters in JSX text content.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags unescaped special characters in JSX text.
#[derive(Debug)]
pub struct NoUnescapedEntities;

/// Characters that must be escaped in JSX text content.
const UNESCAPED_CHARS: &[char] = &['>', '"', '\'', '}'];

impl LintRule for NoUnescapedEntities {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-unescaped-entities".to_owned(),
            description: "Disallow unescaped HTML entities in JSX text".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXText])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXText(text) = node else {
            return;
        };

        let value = text.value.as_str();

        for ch in UNESCAPED_CHARS {
            if value.contains(*ch) {
                let entity = match ch {
                    '>' => "&gt;",
                    '"' => "&quot;",
                    '\'' => "&apos;",
                    '}' => "&#125;",
                    _ => continue,
                };

                // Build fix: replace each occurrence of the char in the JSX text span
                let source = ctx.source_text();
                let text_str = source
                    .get(text.span.start as usize..text.span.end as usize)
                    .unwrap_or("");
                let replaced = text_str.replace(*ch, entity);
                let fix = (replaced != text_str).then(|| Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace `{ch}` with `{entity}`"),
                    edits: vec![Edit {
                        span: Span::new(text.span.start, text.span.end),
                        replacement: replaced,
                    }],
                    is_snippet: false,
                });

                ctx.report(Diagnostic {
                    rule_name: "react/no-unescaped-entities".to_owned(),
                    message: format!(
                        "Unescaped character `{ch}` in JSX text — use `{entity}` instead"
                    ),
                    span: Span::new(text.span.start, text.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace `{ch}` with `{entity}`")),
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoUnescapedEntities, "test.tsx");

    #[test]
    fn test_flags_unescaped_single_quote() {
        let diags = lint(r"const x = <div>it's here</div>;");
        assert!(!diags.is_empty(), "should flag unescaped single quote");
    }

    #[test]
    fn test_flags_unescaped_double_quote() {
        let diags = lint(r#"const x = <div>He said "hello"</div>;"#);
        assert!(!diags.is_empty(), "should flag unescaped quotes");
    }

    #[test]
    fn test_allows_clean_text() {
        let diags = lint(r"const x = <div>hello world</div>;");
        assert!(diags.is_empty(), "clean text should not be flagged");
    }
}
