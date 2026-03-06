//! Rule: `no-document-cookie` (unicorn)
//!
//! Disallow direct use of `document.cookie`. It's error-prone and hard to
//! debug. Use a cookie library or the Cookie Store API instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `document.cookie` usage.
#[derive(Debug)]
pub struct NoDocumentCookie;

impl NativeRule for NoDocumentCookie {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-document-cookie".to_owned(),
            description: "Disallow direct use of `document.cookie`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StaticMemberExpression(member) = kind else {
            return;
        };

        if member.property.name.as_str() != "cookie" {
            return;
        }

        let is_document = matches!(
            &member.object,
            Expression::Identifier(id) if id.name.as_str() == "document"
        );

        if is_document {
            ctx.report(Diagnostic {
                rule_name: "no-document-cookie".to_owned(),
                message: "Do not use `document.cookie` directly — use a cookie library or the Cookie Store API".to_owned(),
                span: Span::new(member.span.start, member.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDocumentCookie)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_document_cookie_read() {
        let diags = lint("var c = document.cookie;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_flags_document_cookie_write() {
        let diags = lint("document.cookie = 'a=b';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_other_property() {
        let diags = lint("var t = document.title;");
        assert!(diags.is_empty());
    }
}
