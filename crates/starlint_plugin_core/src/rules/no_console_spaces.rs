//! Rule: `no-console-spaces`
//!
//! Disallow leading/trailing spaces in `console.log()` string arguments.
//! Leading spaces on the first argument and trailing spaces on the last
//! argument are almost always unintentional formatting mistakes.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Console methods to check.
const CONSOLE_METHODS: &[&str] = &["log", "warn", "error", "info", "debug"];

/// Flags leading/trailing spaces in console method string arguments.
#[derive(Debug)]
pub struct NoConsoleSpaces;

use starlint_ast::node::StringLiteralNode;

impl LintRule for NoConsoleSpaces {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-console-spaces".to_owned(),
            description: "Disallow leading/trailing spaces in console string arguments".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::items_after_statements)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check callee is console.<method>
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };
        let is_console = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "console"
        );
        if !is_console {
            return;
        }
        let method = member.property.as_str();
        if !CONSOLE_METHODS.contains(&method) {
            return;
        }

        if call.arguments.is_empty() {
            return;
        }

        /// Extract a `StringLiteralNode` from a `NodeId` via context.
        fn as_string_literal<'a>(
            id: NodeId,
            ctx: &'a LintContext<'_>,
        ) -> Option<&'a StringLiteralNode> {
            match ctx.node(id) {
                Some(AstNode::StringLiteral(lit)) => Some(lit),
                _ => None,
            }
        }

        let mut edits = Vec::new();

        // Check first argument for leading space
        if let Some(lit) = call
            .arguments
            .first()
            .and_then(|&id| as_string_literal(id, ctx))
        {
            if lit.value.starts_with(' ') && lit.value.len() > 1 {
                edits.push(Edit {
                    span: Span::new(
                        lit.span.start.saturating_add(1),
                        lit.span.start.saturating_add(2),
                    ),
                    replacement: String::new(),
                });
            }
        }

        // Check last argument for trailing space
        if let Some(lit) = call
            .arguments
            .last()
            .and_then(|&id| as_string_literal(id, ctx))
        {
            if lit.value.ends_with(' ') && lit.value.len() > 1 {
                edits.push(Edit {
                    span: Span::new(
                        lit.span.end.saturating_sub(2),
                        lit.span.end.saturating_sub(1),
                    ),
                    replacement: String::new(),
                });
            }
        }

        if edits.is_empty() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-console-spaces".to_owned(),
            message: "Unexpected space in console call argument".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Remove the leading/trailing space from the string".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove space".to_owned(),
                edits,
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConsoleSpaces)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_leading_space() {
        let diags = lint("console.log(' hello');");
        assert_eq!(diags.len(), 1, "should flag leading space");
    }

    #[test]
    fn test_flags_trailing_space() {
        let diags = lint("console.log('hello ');");
        assert_eq!(diags.len(), 1, "should flag trailing space");
    }

    #[test]
    fn test_flags_console_error() {
        let diags = lint("console.error(' oops');");
        assert_eq!(diags.len(), 1, "should flag console.error");
    }

    #[test]
    fn test_flags_console_warn() {
        let diags = lint("console.warn('warning ');");
        assert_eq!(diags.len(), 1, "should flag console.warn");
    }

    #[test]
    fn test_allows_no_spaces() {
        let diags = lint("console.log('hello');");
        assert!(diags.is_empty(), "no spaces should not be flagged");
    }

    #[test]
    fn test_allows_non_string_arg() {
        let diags = lint("console.log(variable);");
        assert!(diags.is_empty(), "non-string arg should not be flagged");
    }

    #[test]
    fn test_allows_non_console() {
        let diags = lint("logger.log(' hello');");
        assert!(diags.is_empty(), "non-console should not be flagged");
    }

    #[test]
    fn test_allows_console_time() {
        let diags = lint("console.time(' timer');");
        assert!(diags.is_empty(), "non-log methods should not be flagged");
    }
}
