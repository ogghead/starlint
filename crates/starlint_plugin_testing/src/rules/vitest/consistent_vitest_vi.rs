//! Rule: `vitest/consistent-vitest-vi`
//!
//! Enforce consistent usage of `vi` instead of `vitest` for mock functions.
//! The `vi` shorthand is the idiomatic way to access Vitest's mock utilities.
//! Using `vitest.fn()`, `vitest.mock()`, or `vitest.spyOn()` should be
//! replaced with `vi.fn()`, `vi.mock()`, or `vi.spyOn()`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/consistent-vitest-vi";

/// Methods on the `vitest` object that should use `vi` instead.
const VI_METHODS: &[&str] = &[
    "fn",
    "mock",
    "spyOn",
    "hoisted",
    "unmock",
    "doMock",
    "doUnmock",
    "importActual",
    "importMock",
    "restoreAllMocks",
    "resetAllMocks",
    "clearAllMocks",
    "useFakeTimers",
    "useRealTimers",
];

/// Enforce using `vi` shorthand instead of `vitest` for mock utilities.
#[derive(Debug)]
pub struct ConsistentVitestVi;

impl LintRule for ConsistentVitestVi {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce using `vi` instead of `vitest` for mock utilities".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // call.callee and member.object are NodeId — resolve them
        let (method_name, obj_span) = {
            let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
                return;
            };
            let method = member.property.clone();
            let obj_id = member.object;
            let Some(AstNode::IdentifierReference(obj)) = ctx.node(obj_id) else {
                return;
            };
            if obj.name.as_str() != "vitest" {
                return;
            }
            (method, obj.span)
        };

        if VI_METHODS.contains(&method_name.as_str()) {
            // Replace the `vitest` identifier with `vi` in the object position
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Use `vi.{method_name}()` instead of `vitest.{method_name}()`"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "Replace `vitest.{method_name}` with `vi.{method_name}`"
                )),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Replace `vitest` with `vi`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(obj_span.start, obj_span.end),
                        replacement: "vi".to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentVitestVi)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_vitest_fn() {
        let source = "const mock = vitest.fn();";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`vitest.fn()` should be flagged");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("vi.fn")),
            "message should suggest `vi.fn()`"
        );
    }

    #[test]
    fn test_flags_vitest_mock() {
        let source = r#"vitest.mock("./module");"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "`vitest.mock()` should be flagged");
    }

    #[test]
    fn test_allows_vi_fn() {
        let source = "const mock = vi.fn();";
        let diags = lint(source);
        assert!(diags.is_empty(), "`vi.fn()` should not be flagged");
    }
}
