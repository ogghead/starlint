//! Rule: `react/jsx-handler-names`
//!
//! Suggest that event handler props (`onClick`, `onChange`, etc.) should
//! reference handler functions starting with `handle`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeName, JSXAttributeValue, JSXExpression};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-handler-names";

/// Suggests that event handler props (names starting with `on`) should reference
/// handler functions named with the `handle` prefix.
#[derive(Debug)]
pub struct JsxHandlerNames;

impl NativeRule for JsxHandlerNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce handler function naming conventions for event props".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        // Check if the prop name starts with "on" followed by an uppercase letter
        let JSXAttributeName::Identifier(ident) = &attr.name else {
            return;
        };
        let prop_name = ident.name.as_str();

        if !prop_name.starts_with("on") {
            return;
        }

        // Make sure the char after "on" is uppercase (e.g., onClick, onChange)
        let Some(after_on) = prop_name.get(2..3) else {
            return;
        };
        if !after_on
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase())
        {
            return;
        }

        // Check if the value is an identifier reference
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return;
        };

        if let JSXExpression::Identifier(ident_ref) = &container.expression {
            let handler_name = ident_ref.name.as_str();
            // The handler should start with "handle" or "on" (passing props through)
            if !handler_name.starts_with("handle") && !handler_name.starts_with("on") {
                ctx.report_warning(
                    RULE_NAME,
                    &format!(
                        "Handler function for `{prop_name}` should be named starting with `handle` (e.g., `handle{}`)",
                        &prop_name[2..]
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxHandlerNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_non_handle_prefix() {
        let diags = lint("const el = <button onClick={doSomething} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag handler not starting with 'handle'"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_handle_prefix() {
        let diags = lint("const el = <button onClick={handleClick} />;");
        assert!(
            diags.is_empty(),
            "should not flag handler starting with 'handle'"
        );
    }

    #[test]
    fn test_allows_on_prefix_passthrough() {
        let diags = lint("const el = <button onClick={onClick} />;");
        assert!(
            diags.is_empty(),
            "should not flag handler starting with 'on' (prop passthrough)"
        );
    }
}
