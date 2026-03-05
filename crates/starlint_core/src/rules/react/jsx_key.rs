//! Rule: `react/jsx-key`
//!
//! Warn when JSX elements in array `.map()` calls are missing a `key` prop.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression, JSXAttributeItem, JSXAttributeName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-key";

/// Warns when JSX elements returned from `.map()` callbacks lack a `key` prop.
#[derive(Debug)]
pub struct JsxKey;

/// Check whether a JSX opening element has a `key` attribute.
fn has_key_prop(attrs: &[JSXAttributeItem<'_>]) -> bool {
    attrs.iter().any(|item| {
        if let JSXAttributeItem::Attribute(attr) = item {
            if let JSXAttributeName::Identifier(ident) = &attr.name {
                return ident.name.as_str() == "key";
            }
        }
        false
    })
}

/// Check whether an expression is a JSX element or fragment without a key.
fn is_jsx_without_key(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::JSXElement(el) => !has_key_prop(&el.opening_element.attributes),
        Expression::JSXFragment(_) => true,
        Expression::ParenthesizedExpression(paren) => is_jsx_without_key(&paren.expression),
        _ => false,
    }
}

/// Check if a callback argument returns JSX without a `key` prop.
fn callback_returns_jsx_without_key(callback: &Argument<'_>) -> bool {
    match callback {
        Argument::ArrowFunctionExpression(arrow) => {
            // Arrow with expression body: `items.map(x => <div />)`
            if arrow.expression {
                if let Some(oxc_ast::ast::Statement::ExpressionStatement(expr_stmt)) =
                    arrow.body.statements.first()
                {
                    return is_jsx_without_key(&expr_stmt.expression);
                }
            }
            // Arrow with block body: check return statements
            for stmt in &arrow.body.statements {
                if let oxc_ast::ast::Statement::ReturnStatement(ret) = stmt {
                    if let Some(ret_val) = &ret.argument {
                        if is_jsx_without_key(ret_val) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        Argument::FunctionExpression(func) => {
            let Some(body) = &func.body else {
                return false;
            };
            for stmt in &body.statements {
                if let oxc_ast::ast::Statement::ReturnStatement(ret) = stmt {
                    if let Some(ret_val) = &ret.argument {
                        if is_jsx_without_key(ret_val) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

impl NativeRule for JsxKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when JSX elements in `.map()` calls are missing a `key` prop"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is `<expr>.map`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "map" {
            return;
        }

        // Check the first argument (the callback)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        if callback_returns_jsx_without_key(first_arg) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Missing `key` prop for JSX element in `.map()` iterator".to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxKey)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_missing_key_in_map() {
        let diags = lint("const items = [1,2].map(x => <div>{x}</div>);");
        assert_eq!(diags.len(), 1, "should flag JSX without key in .map()");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_key_present() {
        let diags = lint("const items = [1,2].map(x => <div key={x}>{x}</div>);");
        assert!(diags.is_empty(), "should not flag when key prop is present");
    }

    #[test]
    fn test_flags_block_body_missing_key() {
        let diags = lint("const items = [1,2].map(x => { return <li>{x}</li>; });");
        assert_eq!(
            diags.len(),
            1,
            "should flag JSX without key in block-body .map()"
        );
    }
}
