//! Rule: `no-plusplus`
//!
//! Disallow the unary operators `++` and `--`. These can be confusing due
//! to automatic semicolon insertion and can be replaced with `+= 1`/`-= 1`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::UpdateOperator;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `++` and `--` unary operators.
#[derive(Debug)]
pub struct NoPlusplus;

impl LintRule for NoPlusplus {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-plusplus".to_owned(),
            description: "Disallow the unary operators `++` and `--`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::UpdateExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::UpdateExpression(update) = node else {
            return;
        };

        let op_str = match update.operator {
            UpdateOperator::Increment => "++",
            UpdateOperator::Decrement => "--",
        };

        let assign_op = match update.operator {
            UpdateOperator::Increment => "+= 1",
            UpdateOperator::Decrement => "-= 1",
        };

        // Extract the argument source text for the fix
        let source = ctx.source_text();
        let arg_text = ctx
            .node(update.argument)
            .and_then(|arg_node| {
                let s = arg_node.span();
                source.get(s.start as usize..s.end as usize)
            })
            .unwrap_or("");

        let replacement = format!("{arg_text} {assign_op}");
        let fix = (!arg_text.is_empty()).then(|| Fix {
            kind: FixKind::SuggestionFix,
            message: format!("Replace `{op_str}` with `{assign_op}`"),
            edits: vec![Edit {
                span: Span::new(update.span.start, update.span.end),
                replacement,
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "no-plusplus".to_owned(),
            message: format!("Unary operator `{op_str}` used"),
            span: Span::new(update.span.start, update.span.end),
            severity: Severity::Warning,
            help: Some(format!("Use `{assign_op}` instead")),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoPlusplus)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_increment() {
        let diags = lint("x++;");
        assert_eq!(diags.len(), 1, "++ should be flagged");
    }

    #[test]
    fn test_flags_decrement() {
        let diags = lint("x--;");
        assert_eq!(diags.len(), 1, "-- should be flagged");
    }

    #[test]
    fn test_flags_prefix_increment() {
        let diags = lint("++x;");
        assert_eq!(diags.len(), 1, "prefix ++ should be flagged");
    }

    #[test]
    fn test_allows_plus_equal() {
        let diags = lint("x += 1;");
        assert!(diags.is_empty(), "+= 1 should not be flagged");
    }
}
