//! Rule: `jsx-a11y/scope`
//!
//! Enforce `scope` attribute is only used on `<th>` elements.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/scope";

#[derive(Debug)]
pub struct Scope;

impl NativeRule for Scope {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce `scope` attribute is only used on `<th>` elements".to_owned(),
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

        let element_name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        // scope is valid on <th>
        if element_name == "th" {
            return;
        }

        let scope_span = opening.attributes.iter().find_map(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                match &attr.name {
                    JSXAttributeName::Identifier(ident) if ident.name.as_str() == "scope" => {
                        Some(Span::new(attr.span.start, attr.span.end))
                    }
                    _ => None,
                }
            } else {
                None
            }
        });

        if let Some(attr_span) = scope_span {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "The `scope` attribute is only valid on `<th>` elements".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    message: "Remove `scope` attribute".to_owned(),
                    edits: vec![Edit {
                        span: attr_span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Scope)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_scope_on_td() {
        let diags = lint(r#"const el = <td scope="col">header</td>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_scope_on_th() {
        let diags = lint(r#"const el = <th scope="col">header</th>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_td_without_scope() {
        let diags = lint(r"const el = <td>data</td>;");
        assert!(diags.is_empty());
    }
}
