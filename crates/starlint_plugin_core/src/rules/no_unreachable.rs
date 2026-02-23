//! Rule: `no-unreachable`
//!
//! Disallow unreachable code after `return`, `throw`, `break`, or `continue`.
//! Code after these statements can never execute and is almost always a mistake.

#![allow(clippy::shadow_reuse, clippy::shadow_unrelated)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags statements that appear after `return`, `throw`, `break`, or `continue`
/// within the same block.
#[derive(Debug)]
pub struct NoUnreachable;

impl LintRule for NoUnreachable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unreachable".to_owned(),
            description: "Disallow unreachable code after return, throw, break, or continue"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::BlockStatement,
            AstNodeType::FunctionBody,
            AstNodeType::Program,
            AstNodeType::SwitchCase,
        ])
    }

    #[allow(clippy::shadow_unrelated)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Check block statements for unreachable code
        let statements: Option<&[NodeId]> = match node {
            AstNode::FunctionBody(body) => Some(&body.statements),
            AstNode::BlockStatement(block) => Some(&block.body),
            AstNode::Program(program) => Some(&program.body),
            AstNode::SwitchCase(case) => Some(&case.consequent),
            _ => None,
        };

        let Some(stmt_ids) = statements else {
            return;
        };

        let mut found_terminator = false;
        for &stmt_id in stmt_ids {
            let Some(stmt) = ctx.node(stmt_id) else {
                continue;
            };
            if found_terminator {
                // Skip function/class declarations — they're hoisted
                if is_hoisted_declaration(stmt) {
                    continue;
                }
                // Skip empty statements
                if matches!(stmt, AstNode::EmptyStatement(_)) {
                    continue;
                }
                let span = stmt.span();
                let span = Span::new(span.start, span.end);
                let fix = Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove unreachable code".to_owned(),
                    edits: vec![Edit {
                        span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                });
                ctx.report(Diagnostic {
                    rule_name: "no-unreachable".to_owned(),
                    message: "Unreachable code".to_owned(),
                    span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
                // Only report the first unreachable statement per block
                break;
            }

            if is_terminator(stmt) {
                found_terminator = true;
            }
        }
    }
}

/// Check if a statement terminates execution flow.
const fn is_terminator(stmt: &AstNode) -> bool {
    matches!(
        stmt,
        AstNode::ReturnStatement(_)
            | AstNode::ThrowStatement(_)
            | AstNode::BreakStatement(_)
            | AstNode::ContinueStatement(_)
    )
}

/// Check if a statement is a hoisted declaration (function/class).
const fn is_hoisted_declaration(stmt: &AstNode) -> bool {
    matches!(stmt, AstNode::Function(_))
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnreachable)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_code_after_return() {
        let diags = lint("function f() { return 1; var x = 2; }");
        assert_eq!(diags.len(), 1, "code after return should be flagged");
    }

    #[test]
    fn test_flags_code_after_throw() {
        let diags = lint("function f() { throw new Error(); var x = 2; }");
        assert_eq!(diags.len(), 1, "code after throw should be flagged");
    }

    #[test]
    fn test_allows_code_before_return() {
        let diags = lint("function f() { var x = 1; return x; }");
        assert!(diags.is_empty(), "code before return should not be flagged");
    }

    #[test]
    fn test_allows_function_after_return() {
        // Function declarations are hoisted
        let diags = lint("function f() { return g(); function g() { return 1; } }");
        assert!(
            diags.is_empty(),
            "function declaration after return should not be flagged (hoisted)"
        );
    }

    #[test]
    fn test_allows_no_terminator() {
        let diags = lint("function f() { var x = 1; var y = 2; }");
        assert!(
            diags.is_empty(),
            "code without terminator should not be flagged"
        );
    }

    #[test]
    fn test_flags_code_after_break() {
        let diags = lint("for (;;) { break; var x = 1; }");
        assert_eq!(diags.len(), 1, "code after break should be flagged");
    }
}
