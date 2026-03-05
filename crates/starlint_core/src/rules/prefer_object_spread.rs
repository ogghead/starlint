//! Rule: `prefer-object-spread`
//!
//! Disallow using `Object.assign()` with an object literal as the first argument.
//! Prefer `{ ...foo }` over `Object.assign({}, foo)`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Object.assign({}, ...)` that can use spread.
#[derive(Debug)]
pub struct PreferObjectSpread;

impl NativeRule for PreferObjectSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-object-spread".to_owned(),
            description: "Disallow using `Object.assign` with object literal first argument"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Must be Object.assign(...)
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "assign" {
            return;
        }

        if !matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Object") {
            return;
        }

        // Must have at least one argument, and the first must be an empty object literal
        if let Some(first_arg) = call.arguments.first() {
            let is_empty_object = match first_arg {
                Argument::ObjectExpression(obj) => obj.properties.is_empty(),
                _ => false,
            };

            if is_empty_object {
                let source = ctx.source_text();
                let mut parts = Vec::new();
                for arg in call.arguments.iter().skip(1) {
                    let s = usize::try_from(arg.span().start).unwrap_or(0);
                    let e = usize::try_from(arg.span().end).unwrap_or(0);
                    let text = source.get(s..e).unwrap_or("");
                    parts.push(format!("...{text}"));
                }
                let replacement = if parts.is_empty() {
                    "{}".to_owned()
                } else {
                    format!("{{ {} }}", parts.join(", "))
                };

                ctx.report(Diagnostic {
                    rule_name: "prefer-object-spread".to_owned(),
                    message: "Use an object spread instead of `Object.assign` with empty object"
                        .to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some("Replace with object spread".to_owned()),
                    fix: Some(Fix {
                        message: "Replace with object spread".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                    }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferObjectSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_assign_empty_first() {
        let diags = lint("var x = Object.assign({}, foo);");
        assert_eq!(
            diags.len(),
            1,
            "Object.assign with empty object first should be flagged"
        );
    }

    #[test]
    fn test_allows_object_assign_non_empty_first() {
        let diags = lint("var x = Object.assign({ a: 1 }, foo);");
        assert!(
            diags.is_empty(),
            "Object.assign with non-empty first should not be flagged"
        );
    }

    #[test]
    fn test_allows_spread_syntax() {
        let diags = lint("var x = { ...foo };");
        assert!(diags.is_empty(), "spread syntax should not be flagged");
    }
}
