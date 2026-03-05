//! Rule: `react/checked-requires-onchange-or-readonly`
//!
//! Warn when `checked` prop is used without `onChange` or `readOnly`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags JSX elements that have a `checked` prop but lack both `onChange`
/// and `readOnly` props. This causes the component to be a read-only
/// controlled input without proper handling.
#[derive(Debug)]
pub struct CheckedRequiresOnchangeOrReadonly;

/// Extract the plain attribute name from a JSX attribute item, if any.
fn attr_name<'a>(item: &'a JSXAttributeItem<'a>) -> Option<&'a str> {
    match item {
        JSXAttributeItem::Attribute(a) => match &a.name {
            JSXAttributeName::Identifier(id) => Some(id.name.as_str()),
            JSXAttributeName::NamespacedName(_) => None,
        },
        JSXAttributeItem::SpreadAttribute(_) => None,
    }
}

impl NativeRule for CheckedRequiresOnchangeOrReadonly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/checked-requires-onchange-or-readonly".to_owned(),
            description: "Require onChange or readOnly when using checked prop".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let mut has_checked = false;
        let mut has_on_change = false;
        let mut has_read_only = false;

        for attr in &opening.attributes {
            if let Some(name) = attr_name(attr) {
                match name {
                    "checked" => has_checked = true,
                    "onChange" => has_on_change = true,
                    "readOnly" => has_read_only = true,
                    _ => {}
                }
            }
        }

        if has_checked && !has_on_change && !has_read_only {
            ctx.report(Diagnostic {
                rule_name: "react/checked-requires-onchange-or-readonly".to_owned(),
                message: "`checked` prop requires `onChange` or `readOnly` to avoid a read-only controlled input".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CheckedRequiresOnchangeOrReadonly)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_checked_without_onchange_or_readonly() {
        let source = "const x = <input checked />;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "checked without onChange or readOnly should be flagged"
        );
    }

    #[test]
    fn test_allows_checked_with_onchange() {
        let source = "const x = <input checked onChange={handleChange} />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "checked with onChange should not be flagged"
        );
    }

    #[test]
    fn test_allows_checked_with_readonly() {
        let source = "const x = <input checked readOnly />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "checked with readOnly should not be flagged"
        );
    }
}
