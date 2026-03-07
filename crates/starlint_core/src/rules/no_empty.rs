//! Rule: `no-empty`
//!
//! Disallow empty block statements. Empty blocks are usually the result of
//! incomplete refactoring.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Returns `true` if the source text inside a block contains a comment.
fn block_has_comment(source: &str, start: u32, end: u32) -> bool {
    let start_idx: usize = start.try_into().unwrap_or(0);
    let end_idx: usize = end.try_into().unwrap_or(0);
    if start_idx >= end_idx || end_idx > source.len() {
        return false;
    }
    let inner = &source[start_idx..end_idx];
    inner.contains("//") || inner.contains("/*")
}

/// Flags empty block statements (e.g. `if (x) {}`).
#[derive(Debug)]
pub struct NoEmpty;

impl LintRule for NoEmpty {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty".to_owned(),
            description: "Disallow empty block statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BlockStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if let AstNode::BlockStatement(block) = node {
            if block.body.is_empty() && !block_has_comment(ctx.source_text(), block.span.start, block.span.end) {
                let span = Span::new(block.span.start, block.span.end);
                ctx.report(Diagnostic {
                    rule_name: "no-empty".to_owned(),
                    message: "Empty block statement".to_owned(),
                    span,
                    severity: Severity::Warning,
                    help: Some("Add a comment inside the block if intentionally empty".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Add `/* empty */` comment".to_owned(),
                        edits: vec![Edit {
                            span,
                            replacement: "{ /* empty */ }".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEmpty)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_block() {
        let diags = lint("if (true) {}");
        assert_eq!(diags.len(), 1, "should find one empty block");
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("no-empty"),
            "rule name should match"
        );
    }

    #[test]
    fn test_ignores_non_empty_block() {
        let diags = lint("if (true) { console.log('hi'); }");
        assert!(
            diags.is_empty(),
            "non-empty block should have no diagnostics"
        );
    }

    #[test]
    fn test_fix_adds_empty_comment() {
        let diags = lint("if (true) {}");
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        let edit = fix.and_then(|f| f.edits.first());
        assert_eq!(
            edit.map(|e| e.replacement.as_str()),
            Some("{ /* empty */ }"),
            "fix should replace block with comment"
        );
    }

    #[test]
    fn test_flags_empty_try_catch() {
        let diags = lint("try { doSomething(); } catch (e) {}");
        assert_eq!(diags.len(), 1, "empty catch block should be flagged");
    }
}
