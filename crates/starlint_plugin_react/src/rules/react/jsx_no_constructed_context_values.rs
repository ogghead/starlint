//! Rule: `react/jsx-no-constructed-context-values`
//!
//! Warn when a `value` prop on a Context Provider contains an inline
//! object or array literal, causing unnecessary re-renders.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-constructed-context-values";

/// Flags inline object/array literals passed as `value` prop to context
/// providers.
#[derive(Debug)]
pub struct JsxNoConstructedContextValues;

/// Check if the JSX element name looks like a Provider.
fn is_provider_name(name: &str) -> bool {
    // Matches `Foo.Provider` (converted to just "Provider" via member expr)
    // or names ending with "Provider" like `MyContextProvider`
    name.ends_with("Provider") || name.contains(".Provider")
}

impl LintRule for JsxNoConstructedContextValues {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow inline constructed values as context provider `value` props"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // Check if this looks like a context provider
        if !is_provider_name(&opening.name) {
            return;
        }

        let attrs: Vec<NodeId> = opening.attributes.to_vec();

        // Look for the `value` attribute
        for &attr_id in &attrs {
            let Some(AstNode::JSXAttribute(attr)) = ctx.node(attr_id) else {
                continue;
            };
            if attr.name != "value" {
                continue;
            }

            let attr_span = attr.span;

            if let Some(value_id) = attr.value {
                // Check if value is a JSXExpressionContainer containing an inline object/array/new
                if let Some(AstNode::JSXExpressionContainer(container)) = ctx.node(value_id) {
                    if let Some(expr_id) = container.expression {
                        let is_constructed = matches!(
                            ctx.node(expr_id),
                            Some(
                                AstNode::ObjectExpression(_)
                                    | AstNode::ArrayExpression(_)
                                    | AstNode::NewExpression(_)
                            )
                        );
                        if is_constructed {
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: "Context provider `value` contains an inline constructed value that will create a new reference on every render. Extract it to a variable or use `useMemo`".to_owned(),
                                span: Span::new(attr_span.start, attr_span.end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(JsxNoConstructedContextValues);

    #[test]
    fn test_flags_inline_object_value() {
        let diags =
            lint("const el = <MyContext.Provider value={{ foo: 1 }}><div /></MyContext.Provider>;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline object literal as context value"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_inline_array_value() {
        let diags = lint("const el = <Ctx.Provider value={[1, 2, 3]}><div /></Ctx.Provider>;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline array literal as context value"
        );
    }

    #[test]
    fn test_allows_variable_value() {
        let diags = lint(
            "const val = { foo: 1 };\nconst el = <Ctx.Provider value={val}><div /></Ctx.Provider>;",
        );
        assert!(
            diags.is_empty(),
            "should not flag variable reference as context value"
        );
    }
}
