//! Rule: `no-iterator`
//!
//! Disallow the use of the `__iterator__` property. This is an obsolete
//! SpiderMonkey-specific extension. Use `Symbol.iterator` instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of the `__iterator__` property.
#[derive(Debug)]
pub struct NoIterator;

impl NativeRule for NoIterator {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-iterator".to_owned(),
            description: "Disallow the `__iterator__` property".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::StaticMemberExpression(member) = kind {
            if member.property.name.as_str() == "__iterator__" {
                // Fix: obj.__iterator__ → obj[Symbol.iterator]
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let obj_span = member.object.span();
                    source
                        .get(obj_span.start as usize..obj_span.end as usize)
                        .map(|obj_text| {
                            let replacement = format!("{obj_text}[Symbol.iterator]");
                            Fix {
                                kind: FixKind::SuggestionFix,
                                message: format!("Replace with `{replacement}`"),
                                edits: vec![Edit {
                                    span: Span::new(member.span.start, member.span.end),
                                    replacement,
                                }],
                                is_snippet: false,
                            }
                        })
                };

                ctx.report(Diagnostic {
                    rule_name: "no-iterator".to_owned(),
                    message: "Use `Symbol.iterator` instead of `__iterator__`".to_owned(),
                    span: Span::new(member.span.start, member.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace `.__iterator__` with `[Symbol.iterator]`".to_owned()),
                    fix,
                    labels: vec![],
                });
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoIterator)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_iterator_property() {
        let diags = lint("foo.__iterator__ = function() {};");
        assert_eq!(diags.len(), 1, "__iterator__ property should be flagged");
    }

    #[test]
    fn test_allows_symbol_iterator() {
        let diags = lint("foo[Symbol.iterator] = function() {};");
        assert!(diags.is_empty(), "Symbol.iterator should not be flagged");
    }

    #[test]
    fn test_allows_normal_property() {
        let diags = lint("foo.bar = 1;");
        assert!(diags.is_empty(), "normal property should not be flagged");
    }
}
