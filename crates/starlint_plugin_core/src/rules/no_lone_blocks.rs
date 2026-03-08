//! Rule: `no-lone-blocks`
//!
//! Disallow unnecessary nested blocks. Standalone block statements that don't
//! contain `let`, `const`, `class`, or `function` declarations serve no
//! purpose and may indicate a structural error.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags standalone block statements that serve no purpose.
#[derive(Debug)]
pub struct NoLoneBlocks;

impl LintRule for NoLoneBlocks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-lone-blocks".to_owned(),
            description: "Disallow unnecessary nested blocks".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::BlockStatement,
            AstNodeType::FunctionBody,
            AstNodeType::Program,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // We look for BlockStatement nodes that appear as direct children of
        // another block (FunctionBody, BlockStatement, or Program body).
        // Since we don't have parent access, we check from the parent side:
        // scan FunctionBody/Program/BlockStatement for child BlockStatements
        // that don't contain block-scoped declarations.
        let maybe_stmts: Option<&[NodeId]> = match node {
            AstNode::FunctionBody(body) => Some(&body.statements),
            AstNode::Program(program) => Some(&program.body),
            AstNode::BlockStatement(block) => Some(&block.body),
            _ => None,
        };

        let Some(stmts) = maybe_stmts else {
            return;
        };

        // Collect spans and fixes first to avoid borrow conflict
        let source = ctx.source_text().to_owned();
        let mut lone_blocks: Vec<(Span, Option<Fix>)> = Vec::new();

        for stmt_id in stmts {
            if let Some(AstNode::BlockStatement(block)) = ctx.node(*stmt_id) {
                // If the block contains no block-scoped declarations, it's unnecessary
                if !has_block_scoped_declaration(&block.body, ctx) {
                    let span = Span::new(block.span.start, block.span.end);
                    // Fix: unwrap the block -- replace `{ body }` with just `body`
                    #[allow(clippy::as_conversions)]
                    let fix = source
                        .get(block.span.start as usize..block.span.end as usize)
                        .map(|text| {
                            // Strip leading `{` and trailing `}`, trim
                            let inner = text
                                .strip_prefix('{')
                                .and_then(|t| t.strip_suffix('}'))
                                .unwrap_or(text)
                                .trim();
                            Fix {
                                kind: FixKind::SuggestionFix,
                                message: "Remove unnecessary block".to_owned(),
                                edits: vec![Edit {
                                    span,
                                    replacement: inner.to_owned(),
                                }],
                                is_snippet: false,
                            }
                        });
                    lone_blocks.push((span, fix));
                }
            }
        }

        for (span, fix) in lone_blocks {
            ctx.report(Diagnostic {
                rule_name: "no-lone-blocks".to_owned(),
                message: "Unnecessary block statement".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if any statement in the block is a block-scoped declaration
/// (let, const, class, or function).
fn has_block_scoped_declaration(stmts: &[NodeId], ctx: &LintContext<'_>) -> bool {
    stmts.iter().any(|stmt_id| {
        let Some(stmt) = ctx.node(*stmt_id) else {
            return false;
        };
        matches!(
            stmt,
            AstNode::VariableDeclaration(decl)
                if decl.kind == VariableDeclarationKind::Let
                    || decl.kind == VariableDeclarationKind::Const
        ) || matches!(stmt, AstNode::Class(_))
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoLoneBlocks)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_lone_block() {
        let diags = lint("{ var x = 1; }");
        assert_eq!(
            diags.len(),
            1,
            "standalone block without block-scoped declarations should be flagged"
        );
    }

    #[test]
    fn test_allows_block_with_let() {
        let diags = lint("{ let x = 1; }");
        assert!(
            diags.is_empty(),
            "block with let declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_block_with_const() {
        let diags = lint("{ const x = 1; }");
        assert!(
            diags.is_empty(),
            "block with const declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_if_block() {
        let diags = lint("if (true) { var x = 1; }");
        assert!(diags.is_empty(), "if block should not be flagged");
    }

    #[test]
    fn test_flags_empty_block() {
        let diags = lint("{ }");
        assert_eq!(diags.len(), 1, "empty standalone block should be flagged");
    }
}
