//! Rule: `no-new`
//!
//! Disallow `new` operators with side effects outside of assignments.
//! Using `new` for side effects (e.g. `new Person()`) without assigning
//! the result is wasteful and confusing.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};

/// Flags `new` expressions used as statements (result not stored).
#[derive(Debug)]
pub struct NoNew;

impl LintRule for NoNew {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-new".to_owned(),
            description: "Disallow `new` for side effects".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExpressionStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ExpressionStatement(stmt) = node else {
            return;
        };

        if matches!(ctx.node(stmt.expression), Some(AstNode::NewExpression(_))) {
            let span = Span::new(stmt.span.start, stmt.span.end);
            let fix = FixBuilder::new("Remove `new` statement", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "no-new".to_owned(),
                message: "Do not use `new` for side effects".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNew)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_as_statement() {
        let diags = lint("new Person();");
        assert_eq!(diags.len(), 1, "new as statement should be flagged");
    }

    #[test]
    fn test_allows_new_assigned() {
        let diags = lint("var p = new Person();");
        assert!(
            diags.is_empty(),
            "new assigned to variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_call_statement() {
        let diags = lint("doSomething();");
        assert!(
            diags.is_empty(),
            "normal function call should not be flagged"
        );
    }
}
