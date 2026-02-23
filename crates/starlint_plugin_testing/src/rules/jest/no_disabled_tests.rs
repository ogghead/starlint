//! Rule: `jest/no-disabled-tests`
//!
//! Warn when `xdescribe`, `xit`, `xtest`, `test.skip`, `it.skip`, `describe.skip` are used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-disabled-tests";

/// Disabled-test prefixed identifiers.
const DISABLED_IDENTIFIERS: &[&str] = &["xdescribe", "xit", "xtest"];

/// Identifiers that can have `.skip` called on them.
const SKIP_BASES: &[&str] = &["describe", "it", "test"];

/// Flags disabled/skipped tests that may be forgotten.
#[derive(Debug)]
pub struct NoDisabledTests;

impl LintRule for NoDisabledTests {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow disabled tests (`xdescribe`, `xtest`, `.skip`)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("xdescribe")
            || source_text.contains("xit")
            || source_text.contains("xtest")
            || source_text.contains(".skip"))
            && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        match ctx.node(call.callee) {
            // xdescribe(...), xit(...), xtest(...)
            Some(AstNode::IdentifierReference(id))
                if DISABLED_IDENTIFIERS.contains(&id.name.as_str()) =>
            {
                let replacement = match id.name.as_str() {
                    "xdescribe" => "describe",
                    "xit" => "it",
                    "xtest" => "test",
                    _ => return,
                };
                let id_span = Span::new(id.span.start, id.span.end);
                let id_name = id.name.clone();
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "Unexpected disabled test: `{id_name}()` — remove or re-enable"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
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
            // describe.skip(...), it.skip(...), test.skip(...)
            Some(AstNode::StaticMemberExpression(member)) => {
                if member.property.as_str() == "skip" {
                    let is_test_base = matches!(
                        ctx.node(member.object),
                        Some(AstNode::IdentifierReference(id)) if SKIP_BASES.contains(&id.name.as_str())
                    );
                    if is_test_base {
                        let base_name = if let Some(AstNode::IdentifierReference(id)) =
                            ctx.node(member.object)
                        {
                            id.name.as_str().to_owned()
                        } else {
                            "test".to_owned()
                        };
                        // Replace `test.skip` with `test` (remove `.skip`)
                        let callee_span = Span::new(member.span.start, member.span.end);
                        ctx.report(Diagnostic {
                            rule_name: RULE_NAME.to_owned(),
                            message: format!(
                                "Unexpected disabled test: `{base_name}.skip()` — remove or re-enable"
                            ),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Warning,
                            help: Some(format!("Remove `.skip` from `{base_name}.skip`")),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: format!("Replace `{base_name}.skip` with `{base_name}`"),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDisabledTests)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_xtest() {
        let diags = lint("xtest('my test', () => {});");
        assert_eq!(diags.len(), 1, "`xtest` should be flagged");
    }

    #[test]
    fn test_flags_it_skip() {
        let diags = lint("it.skip('my test', () => {});");
        assert_eq!(diags.len(), 1, "`it.skip` should be flagged");
    }

    #[test]
    fn test_allows_regular_it() {
        let diags = lint("it('my test', () => {});");
        assert!(diags.is_empty(), "regular `it()` should not be flagged");
    }
}
