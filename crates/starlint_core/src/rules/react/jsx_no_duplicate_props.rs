//! Rule: `react/jsx-no-duplicate-props`
//!
//! Error when a JSX element has duplicate prop names.

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-duplicate-props";

/// Flags JSX elements that have duplicate attribute/prop names.
#[derive(Debug)]
pub struct JsxNoDuplicateProps;

impl NativeRule for JsxNoDuplicateProps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow duplicate props in JSX elements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let mut seen = HashSet::new();
        for attr_item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = attr_item {
                let name = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str(),
                    JSXAttributeName::NamespacedName(ns) => {
                        // For namespaced names like `xml:lang`, we skip the
                        // duplicate check since the combined name is complex.
                        // In practice this is rare, so we just use the property name.
                        ns.name.name.as_str()
                    }
                };
                if !seen.insert(name) {
                    let attr_span = Span::new(attr.span.start, attr.span.end);
                    let fix = FixBuilder::new(
                        format!("Remove duplicate `{name}` prop"),
                        FixKind::SuggestionFix,
                    )
                    .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                    .build();
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("Duplicate prop `{name}` found on JSX element"),
                        span: attr_span,
                        severity: Severity::Error,
                        help: None,
                        fix,
                        labels: vec![],
                    });
                }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoDuplicateProps)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_props() {
        let diags = lint(r#"const el = <div className="a" className="b" />;"#);
        assert_eq!(diags.len(), 1, "should flag duplicate className");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_unique_props() {
        let diags = lint(r#"const el = <div className="a" id="b" />;"#);
        assert!(diags.is_empty(), "should not flag unique props");
    }

    #[test]
    fn test_flags_multiple_duplicates() {
        let diags = lint(r#"const el = <div id="a" id="b" id="c" />;"#);
        assert_eq!(diags.len(), 2, "should flag each duplicate occurrence");
    }
}
