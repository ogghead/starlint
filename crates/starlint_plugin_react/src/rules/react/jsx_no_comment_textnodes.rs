//! Rule: `react/jsx-no-comment-textnodes`
//!
//! Warn when JSX text contains patterns like `// comment` or `/* comment */`
//! which are rendered as visible text rather than being treated as comments.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-comment-textnodes";

/// Flags JSX text nodes that contain comment-like patterns (`//` or `/* */`)
/// which will be rendered as visible text in the output.
#[derive(Debug)]
pub struct JsxNoCommentTextnodes;

impl LintRule for JsxNoCommentTextnodes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow comments from being inserted as text nodes".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXText])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXText(text) = node else {
            return;
        };

        let value = text.value.as_str();

        // Check for line comments: `// ...`
        let has_line_comment = value.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//")
        });

        // Check for block comments: `/* ... */`
        let has_block_comment = value.contains("/*") && value.contains("*/");

        if has_line_comment || has_block_comment {
            // Wrap the text in a JSX expression comment: {/* text */}
            let trimmed = value.trim();
            let comment_text = trimmed
                .trim_start_matches("//")
                .trim_start_matches("/*")
                .trim_end_matches("*/")
                .trim();
            let fix = FixBuilder::new("Wrap in JSX expression comment", FixKind::SafeFix)
                .replace(
                    Span::new(text.span.start, text.span.end),
                    format!("{{/* {comment_text} */}}"),
                )
                .build();

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Comments inside JSX children will be rendered as text. Use JSX expression containers `{/* comment */}` instead".to_owned(),
                span: Span::new(text.span.start, text.span.end),
                severity: Severity::Warning,
                help: Some("Wrap in `{/* ... */}` to make it a proper JSX comment".to_owned()),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxNoCommentTextnodes)];
        lint_source(source, "test.tsx", &rules)
    }

    #[test]
    fn test_flags_line_comment_in_jsx() {
        let diags = lint("const el = <div>// this is a comment</div>;");
        assert_eq!(diags.len(), 1, "should flag line comment in JSX text");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_block_comment_in_jsx() {
        let diags = lint("const el = <div>/* block comment */</div>;");
        assert_eq!(diags.len(), 1, "should flag block comment in JSX text");
    }

    #[test]
    fn test_allows_normal_text() {
        let diags = lint("const el = <div>Hello world</div>;");
        assert!(diags.is_empty(), "should not flag normal text content");
    }
}
