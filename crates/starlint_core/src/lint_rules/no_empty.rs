//! Rule: `no-empty` (unified `LintRule` version)
//!
//! Disallow empty block statements.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags empty block statements.
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
            if block.body.is_empty() && !block_has_comment(ctx.source_text(), block.span) {
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

/// Check if a block span contains a comment.
fn block_has_comment(source: &str, span: starlint_ast::types::Span) -> bool {
    let start = usize::try_from(span.start).unwrap_or(0);
    let end = usize::try_from(span.end).unwrap_or(0);
    if start >= end || end > source.len() {
        return false;
    }
    if let Some(inner) = source.get(start..end) {
        inner.contains("//") || inner.contains("/*")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::*;
    use crate::ast_converter;
    use crate::lint_rule::LintRule;
    use crate::traversal::{LintDispatchTable, traverse_ast_tree};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        let tree = ast_converter::convert(&parsed.program);
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoEmpty)];
        let table = LintDispatchTable::build_from_indices(&rules, &[0]);
        traverse_ast_tree(
            &tree,
            &rules,
            &table,
            &[],
            source,
            Path::new("test.js"),
            None,
        )
    }

    #[test]
    fn flags_empty_block() {
        let diags = lint("if (true) {}");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn allows_non_empty_block() {
        assert!(lint("if (true) { x(); }").is_empty());
    }

    #[test]
    fn allows_block_with_comment() {
        assert!(lint("if (true) { /* intentional */ }").is_empty());
    }

    #[test]
    fn allows_line_comment() {
        assert!(lint("if (true) { // intentional\n}").is_empty());
    }
}
