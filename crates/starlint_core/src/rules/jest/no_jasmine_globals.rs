//! Rule: `jest/no-jasmine-globals`
//!
//! Error when Jasmine globals like `jasmine.createSpy`, `spyOn`, `fail` are used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "jest/no-jasmine-globals";

/// Standalone Jasmine global identifiers that should not be used.
const JASMINE_GLOBALS: &[&str] = &["spyOn", "spyOnProperty", "fail", "pending"];

/// Flags Jasmine-specific globals that should be replaced with Jest equivalents.
#[derive(Debug)]
pub struct NoJasmineGlobals;

impl LintRule for NoJasmineGlobals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow Jasmine globals — use Jest equivalents".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        match ctx.node(call.callee) {
            // Direct calls to spyOn, fail, pending, etc.
            Some(AstNode::IdentifierReference(id))
                if JASMINE_GLOBALS.contains(&id.name.as_str()) =>
            {
                let id_name = id.name.clone();
                // Fix: `spyOn(x, y)` → `jest.spyOn(x, y)`
                #[allow(clippy::as_conversions)]
                let fix = (id_name.as_str() == "spyOn").then(|| {
                    let source = ctx.source_text();
                    let call_text = &source[call.span.start as usize..call.span.end as usize];
                    let replacement = format!("jest.{call_text}");
                    Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `jest.{id_name}`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!(
                        "`{id_name}` is a Jasmine global — use the Jest equivalent instead"
                    ),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: Some(format!("Replace `{id_name}` with Jest equivalent")),
                    fix,
                    labels: vec![],
                });
            }
            // jasmine.createSpy(), jasmine.createSpyObj(), jasmine.any(), etc.
            Some(AstNode::StaticMemberExpression(member)) => {
                let is_jasmine = matches!(
                    ctx.node(member.object),
                    Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "jasmine"
                );
                if is_jasmine {
                    let prop_name = member.property.clone();
                    // Fix: `jasmine.createSpy(...)` → `jest.fn()`
                    let fix = (prop_name.as_str() == "createSpy").then(|| Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Replace with `jest.fn()`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: "jest.fn()".to_owned(),
                        }],
                        is_snippet: false,
                    });

                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "`jasmine.{prop_name}` is a Jasmine API — use Jest equivalents like `jest.fn()` instead"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: Some("Use Jest equivalent".to_owned()),
                        fix,
                        labels: vec![],
                    });
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoJasmineGlobals)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_spy_on() {
        let diags = lint("spyOn(obj, 'method');");
        assert_eq!(diags.len(), 1, "`spyOn` should be flagged");
    }

    #[test]
    fn test_flags_jasmine_create_spy() {
        let diags = lint("jasmine.createSpy('name');");
        assert_eq!(diags.len(), 1, "`jasmine.createSpy` should be flagged");
    }

    #[test]
    fn test_allows_jest_fn() {
        let diags = lint("jest.fn();");
        assert!(diags.is_empty(), "`jest.fn()` should not be flagged");
    }
}
