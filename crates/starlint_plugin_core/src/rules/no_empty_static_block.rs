//! Rule: `no-empty-static-block`
//!
//! Disallow empty static initialization blocks. An empty `static {}` block
//! in a class has no effect and is almost certainly a mistake.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags empty `static {}` blocks in classes.
#[derive(Debug)]
pub struct NoEmptyStaticBlock;

impl LintRule for NoEmptyStaticBlock {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-static-block".to_owned(),
            description: "Disallow empty static initialization blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StaticBlock])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StaticBlock(block) = node else {
            return;
        };

        if block.body.is_empty() {
            ctx.report(Diagnostic {
                rule_name: "no-empty-static-block".to_owned(),
                message: "Unexpected empty static block".to_owned(),
                span: Span::new(block.span.start, block.span.end),
                severity: Severity::Error,
                help: Some("Remove the empty `static {}` block".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove empty static block".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(block.span.start, block.span.end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    starlint_rule_framework::lint_rule_test!(NoEmptyStaticBlock);

    #[test]
    fn test_flags_empty_static_block() {
        let diags = lint("class Foo { static {} }");
        assert_eq!(diags.len(), 1, "empty static block should be flagged");
    }

    #[test]
    fn test_allows_non_empty_static_block() {
        let diags = lint("class Foo { static { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "non-empty static block should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_without_static_block() {
        let diags = lint("class Foo { constructor() {} }");
        assert!(
            diags.is_empty(),
            "class without static block should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_empty_static_blocks() {
        let diags = lint("class Foo { static {} static {} }");
        assert_eq!(
            diags.len(),
            2,
            "two empty static blocks should both be flagged"
        );
    }
}
