//! Rule: `no-unmodified-loop-condition`
//!
//! Flag `while`/`do-while` loops where the condition variable is never
//! modified inside the loop body. This is a common source of infinite loops.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags while loops where the condition variable is not modified in the body.
#[derive(Debug)]
pub struct NoUnmodifiedLoopCondition;

/// Extract a simple identifier name from a test node id (only handles plain identifiers).
fn extract_test_identifier<'a>(test_id: NodeId, ctx: &'a LintContext<'_>) -> Option<&'a str> {
    match ctx.node(test_id)? {
        AstNode::IdentifierReference(id) => Some(id.name.as_str()),
        _ => None,
    }
}

/// Check if the body source text contains patterns that modify the given identifier.
///
/// Looks for assignment operators, increment, and decrement patterns.
fn body_modifies_identifier(source: &str, body_start: usize, body_end: usize, name: &str) -> bool {
    let Some(body_text) = source.get(body_start..body_end) else {
        return true; // If we can't read the body, assume it might be modified
    };

    // Check for patterns like: name =, name +=, name -=, name++, name--, ++name, --name
    let assignment_pattern = format!("{name} =");
    let plus_assign = format!("{name} +=");
    let minus_assign = format!("{name} -=");
    let times_assign = format!("{name} *=");
    let div_assign = format!("{name} /=");
    let postfix_inc = format!("{name}++");
    let postfix_dec = format!("{name}--");
    let prefix_inc = format!("++{name}");
    let prefix_dec = format!("--{name}");

    body_text.contains(&assignment_pattern)
        || body_text.contains(&plus_assign)
        || body_text.contains(&minus_assign)
        || body_text.contains(&times_assign)
        || body_text.contains(&div_assign)
        || body_text.contains(&postfix_inc)
        || body_text.contains(&postfix_dec)
        || body_text.contains(&prefix_inc)
        || body_text.contains(&prefix_dec)
}

impl LintRule for NoUnmodifiedLoopCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unmodified-loop-condition".to_owned(),
            description: "Disallow unmodified loop conditions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::WhileStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::WhileStatement(stmt) = node else {
            return;
        };

        let Some(ident_name) = extract_test_identifier(stmt.test, ctx) else {
            return;
        };

        let Some(body_node) = ctx.node(stmt.body) else {
            return;
        };
        let body_span = body_node.span();
        let body_start = usize::try_from(body_span.start).unwrap_or(0);
        let body_end = usize::try_from(body_span.end).unwrap_or(0);

        if !body_modifies_identifier(ctx.source_text(), body_start, body_end, ident_name) {
            ctx.report(Diagnostic {
                rule_name: "no-unmodified-loop-condition".to_owned(),
                message: format!("`{ident_name}` is not modified in the loop body"),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnmodifiedLoopCondition);

    #[test]
    fn test_flags_unmodified_condition() {
        let diags = lint("while (x) { doSomething(); }");
        assert_eq!(
            diags.len(),
            1,
            "loop where x is never modified should be flagged"
        );
    }

    #[test]
    fn test_allows_modified_by_assignment() {
        let diags = lint("while (x) { x = false; }");
        assert!(
            diags.is_empty(),
            "loop where x is assigned should not be flagged"
        );
    }

    #[test]
    fn test_allows_modified_by_decrement() {
        let diags = lint("while (x) { x--; }");
        assert!(
            diags.is_empty(),
            "loop where x is decremented should not be flagged"
        );
    }

    #[test]
    fn test_allows_modified_by_increment() {
        let diags = lint("while (x) { x++; }");
        assert!(
            diags.is_empty(),
            "loop where x is incremented should not be flagged"
        );
    }

    #[test]
    fn test_skips_complex_condition() {
        // Complex conditions (not a simple identifier) are skipped
        let diags = lint("while (x > 0) { x--; }");
        assert!(diags.is_empty(), "complex condition should not be checked");
    }
}
