//! Rule: `react/jsx-no-constructed-context-values`
//!
//! Warn when a `value` prop on a Context Provider contains an inline
//! object or array literal, causing unnecessary re-renders.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    JSXAttributeName, JSXAttributeValue, JSXElementName, JSXExpression, JSXMemberExpressionObject,
};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-constructed-context-values";

/// Flags inline object/array literals passed as `value` prop to context
/// providers.
#[derive(Debug)]
pub struct JsxNoConstructedContextValues;

/// Check if the JSX element name looks like a `.Provider` member expression.
fn is_provider(name: &JSXElementName<'_>) -> bool {
    if let JSXElementName::MemberExpression(member) = name {
        if member.property.name.as_str() != "Provider" {
            return false;
        }
        if let JSXMemberExpressionObject::IdentifierReference(_) = &member.object {
            return true;
        }
    }
    false
}

/// Check if the element name is an identifier ending with `Provider`.
fn is_provider_identifier(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(ident) => ident.name.as_str().ends_with("Provider"),
        JSXElementName::IdentifierReference(ident) => ident.name.as_str().ends_with("Provider"),
        JSXElementName::MemberExpression(member) => member.property.name.as_str() == "Provider",
        JSXElementName::NamespacedName(_) | JSXElementName::ThisExpression(_) => false,
    }
}

impl NativeRule for JsxNoConstructedContextValues {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow inline constructed values as context provider `value` props"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Check if this looks like a context provider
        if !is_provider(&opening.name) && !is_provider_identifier(&opening.name) {
            return;
        }

        // Look for the `value` attribute
        for attr_item in &opening.attributes {
            let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = attr_item else {
                continue;
            };
            let is_value = match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == "value",
                JSXAttributeName::NamespacedName(_) => false,
            };
            if !is_value {
                continue;
            }

            if let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value {
                let is_constructed = matches!(
                    &container.expression,
                    JSXExpression::ObjectExpression(_)
                        | JSXExpression::ArrayExpression(_)
                        | JSXExpression::NewExpression(_)
                );
                if is_constructed {
                    ctx.report_warning(
                        RULE_NAME,
                        "Context provider `value` contains an inline constructed value that will create a new reference on every render. Extract it to a variable or use `useMemo`",
                        Span::new(attr.span.start, attr.span.end),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoConstructedContextValues)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

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
