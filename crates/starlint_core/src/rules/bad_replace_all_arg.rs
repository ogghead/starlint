//! Rule: `bad-replace-all-arg` (OXC)
//!
//! Catch `.replaceAll()` called with a regex argument that lacks the global
//! flag. `String.prototype.replaceAll` throws a `TypeError` at runtime if
//! given a regex without the `g` flag.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `.replaceAll(/regex/)` without the global flag.
#[derive(Debug)]
pub struct BadReplaceAllArg;

impl LintRule for BadReplaceAllArg {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-replace-all-arg".to_owned(),
            description: "Catch `.replaceAll()` with a regex missing the `g` flag".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for .replaceAll() calls
        let is_replace_all = ctx.node(call.callee).is_some_and(|callee| {
            matches!(
                callee,
                AstNode::StaticMemberExpression(member) if member.property == "replaceAll"
            )
        });

        if !is_replace_all {
            return;
        }

        // Check if the first argument is a regex literal without the `g` flag
        let Some(&first_arg_id) = call.arguments.first() else {
            return;
        };

        let Some(AstNode::RegExpLiteral(re)) = ctx.node(first_arg_id) else {
            return;
        };

        if re.flags.contains('g') {
            return;
        }

        // Build fix: insert `g` flag by replacing regex span with regex+g
        let fix = ctx
            .source_text()
            .get(re.span.start as usize..re.span.end as usize)
            .map(|regex_text| {
                let replacement = format!("{regex_text}g");
                Fix {
                    kind: FixKind::SafeFix,
                    message: "Add the `g` flag".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(re.span.start, re.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }
            });

        ctx.report(Diagnostic {
            rule_name: "bad-replace-all-arg".to_owned(),
            message: "`.replaceAll()` with a regex requires the global (`g`) flag — \
                     this will throw a TypeError at runtime"
                .to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Error,
            help: Some("Add the `g` flag to the regex".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(BadReplaceAllArg)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_regex_without_global() {
        let diags = lint("'hello'.replaceAll(/l/, 'r');");
        assert_eq!(
            diags.len(),
            1,
            "replaceAll with regex without g flag should be flagged"
        );
    }

    #[test]
    fn test_allows_regex_with_global() {
        let diags = lint("'hello'.replaceAll(/l/g, 'r');");
        assert!(
            diags.is_empty(),
            "replaceAll with regex with g flag should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_argument() {
        let diags = lint("'hello'.replaceAll('l', 'r');");
        assert!(
            diags.is_empty(),
            "replaceAll with string argument should not be flagged"
        );
    }
}
