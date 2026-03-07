//! Rule: `jest/no-deprecated-functions`
//!
//! Error when deprecated Jest functions are used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/no-deprecated-functions";

/// Deprecated `jest.*` methods and their replacements.
const DEPRECATED: &[(&str, &str)] = &[
    ("resetModuleRegistry", "jest.resetModules"),
    ("addMatchers", "expect.extend"),
    ("runTimersToTime", "jest.advanceTimersByTime"),
    ("genMockFromModule", "jest.createMockFromModule"),
];

/// Flags usage of deprecated Jest functions.
#[derive(Debug)]
pub struct NoDeprecatedFunctions;

impl LintRule for NoDeprecatedFunctions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow deprecated Jest functions".to_owned(),
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

        // Match `jest.<method>(...)` pattern
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let is_jest = matches!(
            ctx.node(member.object),
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "jest"
        );
        if !is_jest {
            return;
        }

        let method_name = member.property.as_str();

        for &(deprecated_name, replacement) in DEPRECATED {
            if method_name == deprecated_name {
                // replacement is like "jest.resetModules" or "expect.extend"
                // Replace the entire callee (e.g. `jest.addMatchers` -> `expect.extend`)
                let callee_span = Span::new(member.span.start, member.span.end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`jest.{deprecated_name}` is deprecated -- use `{replacement}` instead"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Replace with `{replacement}`")),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: callee_span,
                            replacement: replacement.to_owned(),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDeprecatedFunctions)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_reset_module_registry() {
        let diags = lint("jest.resetModuleRegistry();");
        assert_eq!(
            diags.len(),
            1,
            "`jest.resetModuleRegistry` should be flagged as deprecated"
        );
    }

    #[test]
    fn test_flags_add_matchers() {
        let diags = lint("jest.addMatchers({});");
        assert_eq!(
            diags.len(),
            1,
            "`jest.addMatchers` should be flagged as deprecated"
        );
    }

    #[test]
    fn test_allows_modern_methods() {
        let diags = lint("jest.resetModules();");
        assert!(
            diags.is_empty(),
            "`jest.resetModules` is not deprecated and should not be flagged"
        );
    }
}
