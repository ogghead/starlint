//! Rule: `nextjs/no-head-element`
//!
//! Forbid usage of the `<head>` HTML element. In Next.js, use the `<Head>`
//! component from `next/head` instead for proper SSR support.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-head-element";

/// Flags usage of the `<head>` HTML element.
#[derive(Debug)]
pub struct NoHeadElement;

impl NativeRule for NoHeadElement {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid `<head>` HTML element, use `next/head` `<Head>` instead"
                .to_owned(),
            category: Category::Correctness,
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

        let is_head = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "head",
            _ => false,
        };

        if is_head {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not use `<head>` -- use the `<Head>` component from `next/head` instead"
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoHeadElement)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_head_element() {
        let diags = lint(r"const el = <head><title>Hello</title></head>;");
        assert_eq!(diags.len(), 1, "<head> element should be flagged");
    }

    #[test]
    fn test_allows_head_component() {
        let diags = lint(r"const el = <Head><title>Hello</title></Head>;");
        assert!(diags.is_empty(), "<Head> component should not be flagged");
    }

    #[test]
    fn test_allows_other_elements() {
        let diags = lint(r"const el = <div>hello</div>;");
        assert!(diags.is_empty(), "other elements should not be flagged");
    }
}
