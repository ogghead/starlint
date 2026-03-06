//! Rule: `nextjs/no-script-component-in-head`
//!
//! Forbid `<Script>` component inside `<Head>`. The `next/script` `<Script>`
//! component should not be placed within `next/head` `<Head>`.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXChild, JSXElementName};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-script-component-in-head";

/// Flags `<Script>` components nested inside `<Head>`.
#[derive(Debug)]
pub struct NoScriptComponentInHead;

/// Check if a JSX element name matches the given string, handling both
/// lowercase `Identifier` and `PascalCase` `IdentifierReference` variants.
fn is_element_name(name: &JSXElementName<'_>, target: &str) -> bool {
    match name {
        JSXElementName::Identifier(ident) => ident.name.as_str() == target,
        JSXElementName::IdentifierReference(ident) => ident.name.as_str() == target,
        _ => false,
    }
}

impl NativeRule for NoScriptComponentInHead {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<Script>` inside `<Head>`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        // Check if this is a <Head> element
        if !is_element_name(&element.opening_element.name, "Head") {
            return;
        }

        // Check children for <Script> components
        for child in &element.children {
            if let JSXChild::Element(child_element) = child {
                if is_element_name(&child_element.opening_element.name, "Script") {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: "Do not use `<Script>` inside `<Head>` -- move `<Script>` outside of `<Head>`".to_owned(),
                        span: Span::new(
                            child_element.opening_element.span.start,
                            child_element.opening_element.span.end,
                        ),
                        severity: Severity::Error,
                        help: None,
                        fix: Some(Fix {
                            message: "Remove `<Script>` from `<Head>`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(
                                    child_element.span.start,
                                    child_element.span.end,
                                ),
                                replacement: String::new(),
                            }],
                        }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoScriptComponentInHead)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_script_in_head() {
        let source = r#"const el = <Head><Script src="/script.js" /></Head>;"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "Script in Head should be flagged");
    }

    #[test]
    fn test_allows_script_outside_head() {
        let source =
            r#"const el = <><Head><title>Hi</title></Head><Script src="/script.js" /></>;"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "Script outside Head should pass");
    }

    #[test]
    fn test_allows_lowercase_script_in_head() {
        let source = r#"const el = <Head><script src="/script.js" /></Head>;"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "lowercase script in Head should not be flagged by this rule"
        );
    }
}
