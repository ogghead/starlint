//! Rule: `vue/no-reserved-component-names`
//!
//! Forbid using reserved HTML element names as Vue component names. Using names
//! like `div`, `span`, `button` etc. as component names leads to conflicts.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-reserved-component-names";

/// Reserved HTML element names that should not be used as component names.
const RESERVED_NAMES: &[&str] = &[
    "html",
    "body",
    "base",
    "head",
    "link",
    "meta",
    "style",
    "title",
    "address",
    "article",
    "aside",
    "footer",
    "header",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "main",
    "nav",
    "section",
    "div",
    "span",
    "p",
    "a",
    "br",
    "hr",
    "img",
    "input",
    "button",
    "select",
    "option",
    "textarea",
    "form",
    "label",
    "table",
    "thead",
    "tbody",
    "tr",
    "td",
    "th",
    "ul",
    "ol",
    "li",
    "dl",
    "dt",
    "dd",
    "pre",
    "code",
    "em",
    "strong",
    "small",
    "sub",
    "sup",
    "i",
    "b",
    "u",
    "s",
    "canvas",
    "video",
    "audio",
    "source",
    "iframe",
    "slot",
    "template",
    "component",
    "transition",
    "keep-alive",
    "teleport",
    "suspense",
];

/// Forbid reserved HTML element names as component names.
#[derive(Debug)]
pub struct NoReservedComponentNames;

impl NativeRule for NoReservedComponentNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid reserved HTML element names as component names".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Look for name: 'value' or name: "value" patterns
        let mut search_start = 0;
        while let Some(offset) = source.get(search_start..).and_then(|s| s.find("name:")) {
            let abs_pos = search_start.saturating_add(offset);
            let after_name = source
                .get(abs_pos.saturating_add(5)..)
                .unwrap_or_default()
                .trim_start();

            // Extract the quoted name value
            let (quote, rest) = if let Some(b'\'' | b'"') = after_name.as_bytes().first() {
                let q = after_name.as_bytes().first().copied().unwrap_or(b'"');
                (char::from(q), after_name.get(1..).unwrap_or_default())
            } else {
                search_start = abs_pos.saturating_add(5);
                continue;
            };

            let end_quote = rest.find(quote).unwrap_or(0);
            let name_value = rest.get(..end_quote).unwrap_or_default();
            let lower = name_value.to_ascii_lowercase();

            if RESERVED_NAMES.contains(&lower.as_str()) {
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = start.saturating_add(
                    u32::try_from(5_usize.saturating_add(2).saturating_add(end_quote)).unwrap_or(0),
                );
                ctx.report_warning(
                    RULE_NAME,
                    &format!(
                        "`{name_value}` is a reserved HTML element name and should not be used as a component name"
                    ),
                    Span::new(start, end),
                );
            }

            search_start = abs_pos.saturating_add(5);
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoReservedComponentNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_reserved_name() {
        let source = r#"export default { name: "div" };"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "reserved name should be flagged");
    }

    #[test]
    fn test_allows_custom_name() {
        let source = r#"export default { name: "MyComponent" };"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "custom name should be allowed");
    }

    #[test]
    fn test_flags_reserved_name_case_insensitive() {
        let source = r#"export default { name: "Button" };"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "reserved name (case-insensitive) should be flagged"
        );
    }
}
