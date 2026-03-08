//! Rule: `jest/no-test-prefixes`
//!
//! Suggest using `test.skip`/`test.only` instead of `xtest`/`ftest` prefixes.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-test-prefixes";

/// Mapping of prefixed names to their preferred forms.
const PREFIX_MAP: &[(&str, &str)] = &[
    ("xdescribe", "describe.skip"),
    ("xtest", "test.skip"),
    ("xit", "it.skip"),
    ("fdescribe", "describe.only"),
    ("ftest", "test.only"),
    ("fit", "it.only"),
];

/// Flags shorthand test prefixes like `xtest`/`fit` and suggests `.skip`/`.only`.
#[derive(Debug)]
pub struct NoTestPrefixes;

impl LintRule for NoTestPrefixes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Use `.skip`/`.only` instead of `x`/`f` test prefixes".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        (source_text.contains("xdescribe")
            || source_text.contains("xtest")
            || source_text.contains("xit")
            || source_text.contains("fdescribe")
            || source_text.contains("ftest")
            || source_text.contains("fit"))
            && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        let Some(AstNode::IdentifierReference(id)) = ctx.node(call.callee) else {
            return;
        };

        let callee_name = id.name.as_str();
        let id_span = Span::new(id.span.start, id.span.end);
        let call_span = Span::new(call.span.start, call.span.end);

        for (prefix, replacement) in PREFIX_MAP {
            if callee_name == *prefix {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("Use `{replacement}` instead of `{prefix}`"),
                    span: call_span,
                    severity: Severity::Warning,
                    help: Some(format!("Replace `{prefix}` with `{replacement}`")),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: id_span,
                            replacement: (*replacement).to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoTestPrefixes)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_xtest() {
        let diags = lint("xtest('skipped', () => {});");
        assert_eq!(diags.len(), 1, "`xtest` should be flagged");
    }

    #[test]
    fn test_flags_fit() {
        let diags = lint("fit('focused', () => {});");
        assert_eq!(diags.len(), 1, "`fit` should be flagged");
    }

    #[test]
    fn test_allows_regular_test() {
        let diags = lint("test('normal', () => {});");
        assert!(diags.is_empty(), "regular `test` should not be flagged");
    }
}
