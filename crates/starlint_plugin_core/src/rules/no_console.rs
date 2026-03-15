//! Rule: `no-console`
//!
//! Disallow `console.*` calls. Useful for production code where logging
//! should use a structured logger instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags `console.*` call statements and offers to remove them.
///
/// Matches `ExpressionStatement` (not `CallExpression`) so the fix can cleanly
/// remove the entire statement. Console calls embedded in other expressions
/// (e.g. `const x = console.log(1)`) are not detected -- this is a known
/// limitation, similar to computed access and aliased console.
#[derive(Debug)]
pub struct NoConsole;

impl LintRule for NoConsole {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-console".to_owned(),
            description: "Disallow `console.*` calls".to_owned(),
            category: Category::Suggestion,
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
        let Some(AstNode::CallExpression(call)) = ctx.node(stmt.expression) else {
            return;
        };
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let Some(AstNode::IdentifierReference(ident)) = ctx.node(member.object) else {
            return;
        };
        if ident.name != "console" {
            return;
        }

        let property_name = member.property.as_str();
        let span = Span::new(stmt.span.start, stmt.span.end);
        let fix = FixBuilder::new(
            format!("Remove `console.{property_name}()` statement"),
            FixKind::SuggestionFix,
        )
        .edit(fix_utils::delete_statement(ctx.source_text(), span))
        .build();
        ctx.report(Diagnostic {
            rule_name: "no-console".to_owned(),
            message: format!("Unexpected `console.{property_name}` call"),
            span,
            severity: Severity::Warning,
            help: Some("Remove the `console` call or replace with a logger".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoConsole);

    #[test]
    fn test_flags_console_log() {
        let diags = lint("console.log('hello');");
        assert_eq!(diags.len(), 1, "should flag console.log");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("console.log")),
            "message should mention console.log"
        );
    }

    #[test]
    fn test_flags_console_error() {
        let diags = lint("console.error('fail');");
        assert_eq!(diags.len(), 1, "should flag console.error");
    }

    #[test]
    fn test_fix_removes_statement() {
        let diags = lint("console.log('hello');");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        let edit = fix.and_then(|f| f.edits.first());
        assert_eq!(
            edit.map(|e| e.replacement.as_str()),
            Some(""),
            "fix should remove the statement"
        );
    }

    #[test]
    fn test_ignores_embedded_console_call() {
        // Known limitation: console calls embedded in expressions are not detected
        // because we match ExpressionStatement (needed for clean statement removal).
        let diags = lint("const x = console.log(1);");
        assert!(
            diags.is_empty(),
            "embedded console call is a known false negative"
        );
    }

    #[test]
    fn test_ignores_non_console() {
        let diags = lint("logger.log('hello');");
        assert!(diags.is_empty(), "should not flag logger.log");
    }

    #[test]
    fn test_ignores_computed_console_access() {
        // Known limitation: computed member access like console["log"]() is not detected.
        let diags = lint(r#"console["log"]("hello");"#);
        assert!(
            diags.is_empty(),
            "computed console access is a known false negative"
        );
    }

    #[test]
    fn test_ignores_aliased_console() {
        // Known limitation: `const c = console; c.log()` is not detected.
        let diags = lint("const c = console; c.log('hello');");
        assert!(
            diags.is_empty(),
            "aliased console access is a known false negative"
        );
    }
}
