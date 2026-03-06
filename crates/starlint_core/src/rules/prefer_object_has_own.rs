//! Rule: `prefer-object-has-own`
//!
//! Prefer `Object.hasOwn()` over `Object.prototype.hasOwnProperty.call()`.
//! `Object.hasOwn()` (ES2022) is shorter and more intuitive.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Object.prototype.hasOwnProperty.call()` patterns.
#[derive(Debug)]
pub struct PreferObjectHasOwn;

impl NativeRule for PreferObjectHasOwn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-object-has-own".to_owned(),
            description: "Prefer `Object.hasOwn()` over `Object.prototype.hasOwnProperty.call()`"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for pattern: <something>.hasOwnProperty.call(...)
        // or Object.prototype.hasOwnProperty.call(...)
        let Expression::StaticMemberExpression(outer_member) = &call.callee else {
            return;
        };

        if outer_member.property.name.as_str() != "call" {
            return;
        }

        // The object should be <something>.hasOwnProperty
        let Expression::StaticMemberExpression(inner_member) = &outer_member.object else {
            return;
        };

        if inner_member.property.name.as_str() != "hasOwnProperty" {
            return;
        }

        // Check if it's Object.prototype.hasOwnProperty or {}.hasOwnProperty
        let is_object_prototype = is_object_prototype_pattern(&inner_member.object);
        let is_object_literal = matches!(&inner_member.object, Expression::ObjectExpression(_));

        if is_object_prototype || is_object_literal {
            let source = ctx.source_text();
            let fix = call
                .arguments
                .first()
                .zip(call.arguments.last())
                .map(|(first, last)| {
                    let f_start = usize::try_from(first.span().start).unwrap_or(0);
                    let l_end = usize::try_from(last.span().end).unwrap_or(0);
                    let args_text = source.get(f_start..l_end).unwrap_or("");
                    Fix {
                        kind: FixKind::SafeFix,
                        message: "Replace with `Object.hasOwn()`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: format!("Object.hasOwn({args_text})"),
                        }],
                        is_snippet: false,
                    }
                });

            ctx.report(Diagnostic {
                rule_name: "prefer-object-has-own".to_owned(),
                message:
                    "Use `Object.hasOwn()` instead of `Object.prototype.hasOwnProperty.call()`"
                        .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Replace with `Object.hasOwn()`".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if expression is `Object.prototype`.
fn is_object_prototype_pattern(expr: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };

    if member.property.name.as_str() != "prototype" {
        return false;
    }

    matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Object")
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferObjectHasOwn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_prototype_has_own_property_call() {
        let diags = lint("Object.prototype.hasOwnProperty.call(obj, 'key');");
        assert_eq!(
            diags.len(),
            1,
            "Object.prototype.hasOwnProperty.call() should be flagged"
        );
    }

    #[test]
    fn test_allows_object_has_own() {
        let diags = lint("Object.hasOwn(obj, 'key');");
        assert!(diags.is_empty(), "Object.hasOwn() should not be flagged");
    }

    #[test]
    fn test_allows_direct_has_own_property() {
        let diags = lint("obj.hasOwnProperty('key');");
        assert!(
            diags.is_empty(),
            "direct hasOwnProperty call should not be flagged"
        );
    }
}
