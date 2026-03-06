//! Rule: `react-perf/jsx-no-new-object-as-prop`
//!
//! Warn when object literals are passed as JSX props, causing unnecessary re-renders.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeValue, JSXExpression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react-perf/jsx-no-new-object-as-prop";

/// Warns when object literals (`{}`) are passed directly as JSX prop values.
///
/// Object literals create a new reference on every render, preventing
/// `React.memo` and `shouldComponentUpdate` from bailing out.
#[derive(Debug)]
pub struct JsxNoNewObjectAsProp;

impl NativeRule for JsxNoNewObjectAsProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prevent object literals from being passed as JSX props".to_owned(),
            category: Category::Performance,
            default_severity: Severity::Warning,
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

        if matches!(container.expression, JSXExpression::ObjectExpression(_)) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not pass an object literal as a JSX prop — it creates a new reference on every render".to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoNewObjectAsProp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_literal_prop() {
        let diags = lint(r#"const el = <Foo style={{ color: "red" }} />;"#);
        assert_eq!(diags.len(), 1, "should flag inline object literal prop");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_variable_reference() {
        let diags = lint("const style = {};\nconst el = <Foo style={style} />;");
        assert!(
            diags.is_empty(),
            "should not flag variable reference as prop"
        );
    }

    #[test]
    fn test_flags_multiple_object_props() {
        let diags = lint(r#"const el = <Foo style={{ color: "red" }} data={{ id: 1 }} />;"#);
        assert_eq!(
            diags.len(),
            2,
            "should flag each inline object literal prop"
        );
    }
}
