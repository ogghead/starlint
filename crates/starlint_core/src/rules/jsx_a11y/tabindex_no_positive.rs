//! Rule: `jsx-a11y/tabindex-no-positive`
//!
//! Forbid positive `tabIndex` values.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jsx-a11y/tabindex-no-positive";

#[derive(Debug)]
pub struct TabindexNoPositive;

impl NativeRule for TabindexNoPositive {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid positive `tabIndex` values".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item {
                let is_tabindex = match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "tabIndex",
                    JSXAttributeName::NamespacedName(_) => false,
                };

                if !is_tabindex {
                    continue;
                }

                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    let val = lit.value.as_str();
                    if let Ok(n) = val.parse::<i32>() {
                        if n > 0 {
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: "Avoid positive `tabIndex` values. They disrupt the natural tab order".to_owned(),
                                span: Span::new(opening.span.start, opening.span.end),
                                severity: Severity::Warning,
                                help: None,
                                fix: Some(Fix {
                                    kind: FixKind::SuggestionFix,
                                    message: "Replace with `tabIndex=\"0\"`".to_owned(),
                                    edits: vec![Edit {
                                        span: Span::new(lit.span.start, lit.span.end),
                                        replacement: "\"0\"".to_owned(),
                                    }],
                                    is_snippet: false,
                                }),
                                labels: vec![],
                            });
                        }
                    }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(TabindexNoPositive)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_positive_tabindex() {
        let diags = lint(r#"const el = <div tabIndex="5">content</div>;"#);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_zero_tabindex() {
        let diags = lint(r#"const el = <div tabIndex="0">content</div>;"#);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_negative_tabindex() {
        let diags = lint(r#"const el = <div tabIndex="-1">content</div>;"#);
        assert!(diags.is_empty());
    }
}
