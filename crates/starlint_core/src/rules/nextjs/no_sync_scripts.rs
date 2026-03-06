//! Rule: `nextjs/no-sync-scripts`
//!
//! Forbid synchronous scripts. Scripts without `async` or `defer` block
//! page rendering and hurt performance.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-sync-scripts";

/// Flags `<script src="...">` elements without `async` or `defer`.
#[derive(Debug)]
pub struct NoSyncScripts;

/// Get the attribute name as a string.
fn attr_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(ident) => ident.name.as_str(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.as_str(),
    }
}

impl NativeRule for NoSyncScripts {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid synchronous scripts".to_owned(),
            category: Category::Performance,
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

        // Only check lowercase `<script>` (HTML element)
        let is_script = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "script",
            _ => false,
        };
        if !is_script {
            return;
        }

        // Check if it has a `src` attribute
        let has_src = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                return attr_name(&attr.name) == "src";
            }
            false
        });

        if !has_src {
            return;
        }

        // Check for `async` or `defer` attributes
        let has_async_or_defer = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                let name = attr_name(&attr.name);
                return name == "async" || name == "defer";
            }
            false
        });

        if !has_async_or_defer {
            // Insert `async` after the element name
            let insert_pos = match &opening.name {
                JSXElementName::Identifier(ident) => ident.span.end,
                _ => opening.span.end,
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Synchronous scripts block page rendering -- add `async` or `defer` to `<script>` elements".to_owned(),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    message: "Add `async` attribute".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(insert_pos, insert_pos),
                        replacement: " async".to_owned(),
                    }],
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSyncScripts)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_sync_script() {
        let diags = lint(r#"const el = <script src="/script.js" />;"#);
        assert_eq!(diags.len(), 1, "sync script should be flagged");
    }

    #[test]
    fn test_allows_async_script() {
        let diags = lint(r#"const el = <script src="/script.js" async />;"#);
        assert!(diags.is_empty(), "async script should pass");
    }

    #[test]
    fn test_allows_defer_script() {
        let diags = lint(r#"const el = <script src="/script.js" defer />;"#);
        assert!(diags.is_empty(), "defer script should pass");
    }
}
