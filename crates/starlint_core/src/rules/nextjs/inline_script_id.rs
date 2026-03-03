//! Rule: `nextjs/inline-script-id`
//!
//! Require `id` attribute on inline `<Script>` components from `next/script`.
//! Next.js uses the `id` to deduplicate inline scripts.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/inline-script-id";

/// Flags inline `<Script>` components missing an `id` attribute.
#[derive(Debug)]
pub struct InlineScriptId;

/// Get the attribute name as a string.
fn attr_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(ident) => ident.name.as_str(),
        JSXAttributeName::NamespacedName(ns) => ns.name.name.as_str(),
    }
}

impl NativeRule for InlineScriptId {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `id` attribute on inline `<Script>` components".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
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

        // Only check `<Script>` (PascalCase -- the Next.js component)
        let is_script = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "Script",
            JSXElementName::IdentifierReference(ident) => ident.name.as_str() == "Script",
            _ => false,
        };
        if !is_script {
            return;
        }

        // Check if it has dangerouslySetInnerHTML
        let has_dangerous = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                return attr_name(&attr.name) == "dangerouslySetInnerHTML";
            }
            false
        });

        // Check if it has a `src` attribute (external script)
        let has_src = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                return attr_name(&attr.name) == "src";
            }
            false
        });

        // An inline script has children or dangerouslySetInnerHTML but no src
        let is_inline = (!element.children.is_empty() || has_dangerous) && !has_src;

        if !is_inline {
            return;
        }

        // Require `id` attribute on inline scripts
        let has_id = opening.attributes.iter().any(|item| {
            if let JSXAttributeItem::Attribute(attr) = item {
                return attr_name(&attr.name) == "id";
            }
            false
        });

        if !has_id {
            ctx.report_error(
                RULE_NAME,
                "Inline `<Script>` components require an `id` attribute for deduplication",
                Span::new(opening.span.start, opening.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(InlineScriptId)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_inline_script_without_id() {
        let diags = lint(r#"const el = <Script>{`console.log("hi")`}</Script>;"#);
        assert_eq!(diags.len(), 1, "inline Script without id should be flagged");
    }

    #[test]
    fn test_allows_inline_script_with_id() {
        let diags = lint(r#"const el = <Script id="my-script">{`console.log("hi")`}</Script>;"#);
        assert!(diags.is_empty(), "inline Script with id should pass");
    }

    #[test]
    fn test_allows_external_script() {
        let diags = lint(r#"const el = <Script src="/script.js"></Script>;"#);
        assert!(diags.is_empty(), "external Script should not require id");
    }
}
