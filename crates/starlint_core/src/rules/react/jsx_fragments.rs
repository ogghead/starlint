//! Rule: `react/jsx-fragments`
//!
//! Suggest using `<>` short syntax instead of `<React.Fragment>` when no key
//! prop is present.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName, JSXMemberExpressionObject};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-fragments";

/// Suggests using `<>` shorthand instead of `<React.Fragment>` when no `key`
/// prop is present.
#[derive(Debug)]
pub struct JsxFragments;

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

/// Check if the element name is `React.Fragment`.
fn is_react_fragment(name: &JSXElementName<'_>) -> bool {
    if let JSXElementName::MemberExpression(member) = name {
        if member.property.name.as_str() != "Fragment" {
            return false;
        }
        if let JSXMemberExpressionObject::IdentifierReference(obj) = &member.object {
            return obj.name.as_str() == "React";
        }
    }
    false
}

impl NativeRule for JsxFragments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `<>` shorthand over `<React.Fragment>` when no key is needed"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
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

        if !is_react_fragment(&opening.name) {
            return;
        }

        // If there's a `key` prop, React.Fragment is required
        if has_key_prop(&opening.attributes) {
            return;
        }

        let opening_span = Span::new(opening.span.start, opening.span.end);

        // Build edits: replace opening tag and closing tag
        let mut edits = vec![Edit {
            span: opening_span,
            replacement: "<>".to_owned(),
        }];

        if let Some(closing) = &element.closing_element {
            edits.push(Edit {
                span: Span::new(closing.span.start, closing.span.end),
                replacement: "</>".to_owned(),
            });
        }

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Prefer `<>` shorthand over `<React.Fragment>` when no `key` prop is needed"
                .to_owned(),
            span: Span::new(element.span.start, element.span.end),
            severity: Severity::Warning,
            help: Some("Replace `<React.Fragment>` with `<>`".to_owned()),
            fix: Some(Fix {
                message: "Replace with shorthand fragment syntax".to_owned(),
                edits,
                is_snippet: false,
            }),
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxFragments)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_react_fragment_without_key() {
        let diags = lint("const el = <React.Fragment><div /></React.Fragment>;");
        assert_eq!(
            diags.len(),
            1,
            "should flag React.Fragment without key prop"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_react_fragment_with_key() {
        let diags = lint("const el = <React.Fragment key=\"k\"><div /></React.Fragment>;");
        assert!(
            diags.is_empty(),
            "should not flag React.Fragment with key prop"
        );
    }

    #[test]
    fn test_allows_short_syntax() {
        let diags = lint("const el = <><div /></>;");
        assert!(
            diags.is_empty(),
            "should not flag shorthand fragment syntax"
        );
    }
}
