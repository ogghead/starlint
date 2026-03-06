//! Rule: `no-proto`
//!
//! Disallow the use of the `__proto__` property. Use `Object.getPrototypeOf`
//! and `Object.setPrototypeOf` instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of the deprecated `__proto__` property.
#[derive(Debug)]
pub struct NoProto;

impl NativeRule for NoProto {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-proto".to_owned(),
            description: "Disallow the use of the `__proto__` property".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::StaticMemberExpression(member) = kind {
            if member.property.name.as_str() == "__proto__" {
                #[allow(clippy::as_conversions)]
                let fix = {
                    let source = ctx.source_text();
                    let obj_span = member.object.span();
                    source
                        .get(obj_span.start as usize..obj_span.end as usize)
                        .map(|obj_text| {
                            let replacement = format!("Object.getPrototypeOf({obj_text})");
                            Fix {
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
                    rule_name: "no-proto".to_owned(),
                    message:
                        "Use `Object.getPrototypeOf`/`Object.setPrototypeOf` instead of `__proto__`"
                            .to_owned(),
                    span: Span::new(member.span.start, member.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace `.__proto__` with `Object.getPrototypeOf()`".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoProto)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_proto_access() {
        let diags = lint("var p = obj.__proto__;");
        assert_eq!(diags.len(), 1, "__proto__ access should be flagged");
    }

    #[test]
    fn test_allows_get_prototype_of() {
        let diags = lint("var p = Object.getPrototypeOf(obj);");
        assert!(
            diags.is_empty(),
            "Object.getPrototypeOf should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_property() {
        let diags = lint("var x = obj.foo;");
        assert!(
            diags.is_empty(),
            "normal property access should not be flagged"
        );
    }
}
