//! Rule: `prefer-structured-clone` (unicorn)
//!
//! Prefer `structuredClone()` over `JSON.parse(JSON.stringify())` for
//! deep cloning objects. `structuredClone` is more efficient and handles
//! more data types correctly.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `JSON.parse(JSON.stringify(x))` patterns.
#[derive(Debug)]
pub struct PreferStructuredClone;

impl NativeRule for PreferStructuredClone {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-structured-clone".to_owned(),
            description: "Prefer structuredClone over JSON.parse(JSON.stringify())".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for JSON.parse(...)
        if !is_json_method_call(&call.callee, "parse") {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        // The argument must be JSON.stringify(...)
        let Some(arg) = call.arguments.first() else {
            return;
        };

        let is_json_stringify = match arg {
            oxc_ast::ast::Argument::CallExpression(inner_call) => {
                is_json_method_call(&inner_call.callee, "stringify")
                    && inner_call.arguments.len() == 1
            }
            _ => false,
        };

        if is_json_stringify {
            // Extract the inner argument text for the fix
            let fix = if let Some(oxc_ast::ast::Argument::CallExpression(inner_call)) =
                call.arguments.first()
            {
                if let Some(inner_arg) = inner_call.arguments.first() {
                    let inner_span = inner_arg.span();
                    let source = ctx.source_text();
                    let arg_text = source
                        .get(inner_span.start as usize..inner_span.end as usize)
                        .unwrap_or("")
                        .to_owned();
                    (!arg_text.is_empty()).then(|| Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `structuredClone({arg_text})`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement: format!("structuredClone({arg_text})"),
                        }],
                        is_snippet: false,
                    })
                } else {
                    None
                }
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "prefer-structured-clone".to_owned(),
                message: "Prefer `structuredClone(x)` over `JSON.parse(JSON.stringify(x))`"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some("Use `structuredClone()` instead".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is `JSON.methodName`.
fn is_json_method_call(expr: &Expression<'_>, method: &str) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };

    let Expression::Identifier(obj) = &member.object else {
        return false;
    };

    obj.name == "JSON" && member.property.name == method
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStructuredClone)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_json_parse_stringify() {
        let diags = lint("var copy = JSON.parse(JSON.stringify(obj));");
        assert_eq!(
            diags.len(),
            1,
            "JSON.parse(JSON.stringify()) should be flagged"
        );
    }

    #[test]
    fn test_allows_structured_clone() {
        let diags = lint("var copy = structuredClone(obj);");
        assert!(diags.is_empty(), "structuredClone should not be flagged");
    }

    #[test]
    fn test_allows_json_parse_alone() {
        let diags = lint("var data = JSON.parse(text);");
        assert!(diags.is_empty(), "JSON.parse alone should not be flagged");
    }

    #[test]
    fn test_allows_json_stringify_alone() {
        let diags = lint("var text = JSON.stringify(obj);");
        assert!(
            diags.is_empty(),
            "JSON.stringify alone should not be flagged"
        );
    }
}
