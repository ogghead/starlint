//! Rule: `react-perf/jsx-no-jsx-as-prop`
//!
//! Warn when JSX elements are passed inline as props, causing unnecessary re-renders.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeValue, JSXExpression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-jsx-as-prop";

/// Warns when JSX elements are passed inline as prop values.
///
/// Inline JSX creates a new element on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
/// Extract the JSX to a variable or memoize it with `useMemo` instead.
#[derive(Debug)]
pub struct JsxNoJsxAsProp;

impl NativeRule for JsxNoJsxAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent JSX elements from being passed inline as props".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXAttribute])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        // Check ExpressionContainer values: `prop={<Child />}`
        if let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value {
            if matches!(
                container.expression,
                JSXExpression::JSXElement(_) | JSXExpression::JSXFragment(_)
            ) {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message:
                        "Do not pass JSX as a prop value — it creates a new element on every render"
                            .to_owned(),
                    span: Span::new(container.span.start, container.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                return;
            }
        }

        // Check direct element values: `prop=<Child />`
        // (valid JSX syntax: `<Foo bar=<Baz /> />`)
        if matches!(
            attr.value,
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_))
        ) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not pass JSX as a prop value — it creates a new element on every render"
                        .to_owned(),
                span: Span::new(attr.span.start, attr.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoJsxAsProp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_jsx_element_in_expression_container() {
        let diags = lint("const el = <Foo icon={<Icon />} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline JSX element passed as prop"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_jsx_fragment_in_expression_container() {
        let diags = lint("const el = <Foo content={<>hello</>} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline JSX fragment passed as prop"
        );
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const icon = <Icon />;\nconst el = <Foo icon={icon} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }
}
