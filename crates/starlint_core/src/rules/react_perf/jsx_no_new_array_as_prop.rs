//! Rule: `react-perf/jsx-no-new-array-as-prop`
//!
//! Warn when array literals are passed as JSX props, causing unnecessary re-renders.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeValue, JSXExpression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-new-array-as-prop";

/// Warns when array literals (`[]`) are passed directly as JSX prop values.
///
/// Array literals create a new reference on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
#[derive(Debug)]
pub struct JsxNoNewArrayAsProp;

impl NativeRule for JsxNoNewArrayAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent array literals from being passed as JSX props".to_owned(),
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

        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return;
        };

        if matches!(container.expression, JSXExpression::ArrayExpression(_)) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not pass an array literal as a JSX prop — it creates a new reference on every render".to_owned(),
                span: Span::new(container.span.start, container.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoNewArrayAsProp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_array_literal_prop() {
        let diags = lint("const el = <Foo items={[1, 2, 3]} />;");
        assert_eq!(diags.len(), 1, "should flag inline array literal prop");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const items = [1, 2];\nconst el = <Foo items={items} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }

    #[test]
    fn test_flags_empty_array_prop() {
        let diags = lint("const el = <Foo items={[]} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag even an empty inline array literal"
        );
    }
}
