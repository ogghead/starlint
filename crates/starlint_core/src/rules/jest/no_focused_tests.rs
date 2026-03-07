//! Rule: `jest/no-focused-tests`
//!
//! Error when `fdescribe`, `fit`, `test.only`, `it.only`, `describe.only` are used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/no-focused-tests";

/// Focused-test prefixed identifiers.
const FOCUSED_IDENTIFIERS: &[&str] = &["fdescribe", "fit"];

/// Identifiers that can have `.only` called on them.
const ONLY_BASES: &[&str] = &["describe", "it", "test"];

/// Flags focused tests that would cause other tests to be skipped in CI.
#[derive(Debug)]
pub struct NoFocusedTests;

impl LintRule for NoFocusedTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow focused tests (`.only`, `fdescribe`, `fit`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        match ctx.node(call.callee) {
            // fdescribe(...) or fit(...)
            Some(AstNode::IdentifierReference(id))
                if FOCUSED_IDENTIFIERS.contains(&id.name.as_str()) =>
            {
                let replacement = match id.name.as_str() {
                    "fdescribe" => "describe",
                    "fit" => "it",
                    _ => return,
                };
                let id_span = Span::new(id.span.start, id.span.end);
                let id_name = id.name.clone();
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Unexpected focused test: `{id_name}()` will prevent other tests from running"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Replace `{id_name}` with `{replacement}`")),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: id_span,
                            replacement: replacement.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
            // describe.only(...), it.only(...), test.only(...)
            Some(AstNode::StaticMemberExpression(member)) => {
                if member.property.as_str() == "only" {
                    let is_test_base = matches!(
                        ctx.node(member.object),
                        Some(AstNode::IdentifierReference(id)) if ONLY_BASES.contains(&id.name.as_str())
                    );
                    if is_test_base {
                        let base_name = if let Some(AstNode::IdentifierReference(id)) =
                            ctx.node(member.object)
                        {
                            id.name.as_str().to_owned()
                        } else {
                            "test".to_owned()
                        };
                        // Replace `test.only` with `test` (remove `.only`)
                        let callee_span = Span::new(member.span.start, member.span.end);
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "Unexpected focused test: `{base_name}.only()` will prevent other tests from running"
                            ),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Error,
                            help: Some(format!("Remove `.only` from `{base_name}.only`")),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: format!("Replace `{base_name}.only` with `{base_name}`"),
                                edits: vec![Edit {
                                    span: callee_span,
                                    replacement: base_name,
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoFocusedTests)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_fdescribe() {
        let diags = lint("fdescribe('suite', () => {});");
        assert_eq!(diags.len(), 1, "`fdescribe` should be flagged");
    }

    #[test]
    fn test_flags_test_only() {
        let diags = lint("test.only('my test', () => {});");
        assert_eq!(diags.len(), 1, "`test.only` should be flagged");
    }

    #[test]
    fn test_allows_regular_test() {
        let diags = lint("test('my test', () => {});");
        assert!(diags.is_empty(), "regular `test()` should not be flagged");
    }
}
