//! Rule: `jsx-a11y/label-has-associated-control`
//!
//! Enforce `<label>` elements have an associated control via `htmlFor` or nesting.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/label-has-associated-control";

#[derive(Debug)]
pub struct LabelHasAssociatedControl;

/// Check if an attribute exists on a JSX element.
fn has_attribute(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> bool {
    opening.attributes.iter().any(|item| {
        if let JSXAttributeItem::Attribute(attr) = item {
            match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str() == name,
                JSXAttributeName::NamespacedName(_) => false,
            }
        } else {
            false
        }
    })
}

impl NativeRule for LabelHasAssociatedControl {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Enforce `<label>` elements have an associated control via `htmlFor` or nesting"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        let opening = &element.opening_element;

        let is_label = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "label",
            _ => false,
        };

        if !is_label {
            return;
        }

        // Check for htmlFor attribute
        let has_html_for = has_attribute(opening, "htmlFor");

        // If no children and no htmlFor, the label has no associated control
        if element.children.is_empty() && !has_html_for {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "`<label>` must have an associated control via `htmlFor` or by nesting an input"
                        .to_owned(),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(LabelHasAssociatedControl)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bare_label_without_htmlfor() {
        let diags = lint(r"const el = <label />;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_label_with_htmlfor() {
        let diags = lint(r#"const el = <label htmlFor="input-id">Name</label>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_label_with_children() {
        let diags = lint(r"const el = <label>Name <input /></label>;");
        assert!(diags.is_empty());
    }
}
