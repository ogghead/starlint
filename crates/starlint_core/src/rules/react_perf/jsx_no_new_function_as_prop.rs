//! Rule: `react-perf/jsx-no-new-function-as-prop`
//!
//! Warn when inline functions are passed as JSX props, causing unnecessary re-renders.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeValue, JSXExpression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-new-function-as-prop";

/// Warns when inline functions (arrow functions or function expressions) are
/// passed directly as JSX prop values.
///
/// Inline functions create a new closure on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
/// Use `useCallback` or define the handler outside the render path instead.
#[derive(Debug)]
pub struct JsxNoNewFunctionAsProp;

impl NativeRule for JsxNoNewFunctionAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent inline functions from being passed as JSX props".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXAttribute])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return;
        };

        let is_inline_function = matches!(
            container.expression,
            JSXExpression::ArrowFunctionExpression(_) | JSXExpression::FunctionExpression(_)
        );

        if is_inline_function {
            ctx.report_warning(
                RULE_NAME,
                "Do not pass an inline function as a JSX prop — it creates a new closure on every render",
                Span::new(container.span.start, container.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoNewFunctionAsProp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_arrow_function_prop() {
        let diags = lint("const el = <Foo onClick={() => console.log('click')} />;");
        assert_eq!(diags.len(), 1, "should flag inline arrow function prop");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_function_expression_prop() {
        let diags = lint("const el = <Foo onClick={function() { return 1; }} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag inline function expression prop"
        );
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const handler = () => {};\nconst el = <Foo onClick={handler} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }
}
