//! Rule: `typescript/parameter-properties`
//!
//! Disallow TypeScript parameter properties in class constructors. Parameter
//! properties (e.g. `constructor(public name: string)`) combine parameter
//! declaration and property assignment into one, which can be confusing and
//! makes class structure harder to read at a glance.

#![allow(clippy::or_fun_call)]
use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::MethodDefinitionKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/parameter-properties";

/// Flags constructor parameters that use TypeScript parameter properties
/// (accessibility modifiers or `readonly`).
#[derive(Debug)]
pub struct ParameterProperties;

impl LintRule for ParameterProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow TypeScript parameter properties in class constructors"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::MethodDefinition])
    }

    #[allow(clippy::map_unwrap_or)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::MethodDefinition(method) = node else {
            return;
        };

        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        // Resolve the method value (function node) to access its params.
        // Since the flat AST doesn't preserve parameter property annotations
        // (accessibility, readonly), we use a source text heuristic.
        let func_body_span = match ctx.node(method.value) {
            Some(AstNode::Function(func)) => {
                // Check the source text of the constructor signature for parameter properties
                func.body
                    .and_then(|body_id| ctx.node(body_id).map(starlint_ast::AstNode::span))
            }
            _ => return,
        };

        // Use the source text to detect parameter properties in the constructor
        let source = ctx.source_text();
        let method_start = usize::try_from(method.span.start).unwrap_or(0);
        let body_start = func_body_span
            .map_or(usize::try_from(method.span.end).unwrap_or(0), |s| {
                usize::try_from(s.start).unwrap_or(0)
            });
        let signature = source.get(method_start..body_start).unwrap_or("");

        // Check for parameter property keywords in the constructor signature
        let pp_keywords = ["public ", "private ", "protected ", "readonly "];
        for keyword in &pp_keywords {
            if signature.contains(keyword) {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Unexpected parameter property — declare the property explicitly in the class body instead".to_owned(),
                    span: Span::new(method.span.start, method.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
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

    starlint_rule_framework::lint_rule_test!(ParameterProperties, "test.ts");

    #[test]
    fn test_flags_public_parameter_property() {
        let diags = lint("class Foo { constructor(public name: string) {} }");
        assert_eq!(
            diags.len(),
            1,
            "public parameter property should be flagged"
        );
    }

    #[test]
    fn test_flags_private_parameter_property() {
        let diags = lint("class Foo { constructor(private name: string) {} }");
        assert_eq!(
            diags.len(),
            1,
            "private parameter property should be flagged"
        );
    }

    #[test]
    fn test_flags_readonly_parameter_property() {
        let diags = lint("class Foo { constructor(readonly name: string) {} }");
        assert_eq!(
            diags.len(),
            1,
            "readonly parameter property should be flagged"
        );
    }

    #[test]
    fn test_allows_plain_constructor_parameter() {
        let diags = lint("class Foo { constructor(name: string) {} }");
        assert!(
            diags.is_empty(),
            "plain constructor parameter should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_constructor_method() {
        let diags = lint("class Foo { bar(name: string) {} }");
        assert!(
            diags.is_empty(),
            "non-constructor method parameter should not be flagged"
        );
    }
}
