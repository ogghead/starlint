//! Rule: `react/jsx-boolean-value`
//!
//! Suggest omitting `={true}` for boolean JSX attributes.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeName, JSXAttributeValue, JSXExpression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-boolean-value";

/// Suggests omitting `={true}` from boolean JSX attributes since
/// `<Comp disabled />` is equivalent to `<Comp disabled={true} />`.
#[derive(Debug)]
pub struct JsxBooleanValue;

impl NativeRule for JsxBooleanValue {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce omitting `={true}` for boolean JSX attributes".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return;
        };

        if let JSXExpression::BooleanLiteral(lit) = &container.expression {
            if lit.value {
                let prop_name = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str(),
                    JSXAttributeName::NamespacedName(ns) => ns.name.name.as_str(),
                };
                ctx.report_warning(
                    RULE_NAME,
                    &format!(
                        "Unnecessary `={{true}}` for boolean attribute `{prop_name}` — just use `{prop_name}`"
                    ),
                    Span::new(attr.span.start, attr.span.end),
                );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxBooleanValue)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_explicit_true() {
        let diags = lint("const el = <button disabled={true} />;");
        assert_eq!(diags.len(), 1, "should flag explicit true value");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_shorthand() {
        let diags = lint("const el = <button disabled />;");
        assert!(diags.is_empty(), "should not flag shorthand boolean");
    }

    #[test]
    fn test_allows_explicit_false() {
        let diags = lint("const el = <button disabled={false} />;");
        assert!(
            diags.is_empty(),
            "should not flag explicit false (it's necessary)"
        );
    }
}
