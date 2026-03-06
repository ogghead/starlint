//! Rule: `prefer-set-size`
//!
//! Prefer `Set#size` over converting a Set to an array and checking `.length`.
//! Patterns like `[...set].length` or `Array.from(set).length` create an
//! unnecessary intermediate array just to count elements.

use oxc_ast::AstKind;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.length` access on patterns that convert a Set to an array.
#[derive(Debug)]
pub struct PreferSetSize;

impl NativeRule for PreferSetSize {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-set-size".to_owned(),
            description: "Prefer `Set#size` over converting to array and checking `.length`"
                .to_owned(),
            category: Category::Suggestion,
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

        if member.property.name.as_str() != "length" {
            return;
        }

        // Pattern 1: `[...x].length` — array with a single spread element
        if let Some(set_name) = get_spread_arg_name(&member.object, ctx.source_text()) {
            let replacement = format!("{set_name}.size");
            ctx.report(Diagnostic {
                rule_name: "prefer-set-size".to_owned(),
                message: "Use `Set#size` instead of spreading into an array and checking `.length`"
                    .to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(member.span.start, member.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
            return;
        }

        // Pattern 2: `Array.from(x).length` — call to Array.from with one argument
        if let Some(set_name) = get_array_from_arg_name(&member.object, ctx.source_text()) {
            let replacement = format!("{set_name}.size");
            ctx.report(Diagnostic {
                rule_name: "prefer-set-size".to_owned(),
                message: "Use `Set#size` instead of `Array.from()` and `.length`".to_owned(),
                span: Span::new(member.span.start, member.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(member.span.start, member.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Extract the spread argument name from `[...something]` (array with a single spread element).
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn get_spread_arg_name<'s>(expr: &Expression<'_>, source: &'s str) -> Option<&'s str> {
    let Expression::ArrayExpression(array) = expr else {
        return None;
    };

    if array.elements.len() != 1 {
        return None;
    }

    let Some(ArrayExpressionElement::SpreadElement(spread)) = array.elements.first() else {
        return None;
    };

    let span = spread.argument.span();
    Some(&source[span.start as usize..span.end as usize])
}

/// Extract the argument name from `Array.from(something)` (single-argument call).
#[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
fn get_array_from_arg_name<'s>(expr: &Expression<'_>, source: &'s str) -> Option<&'s str> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };

    if call.arguments.len() != 1 {
        return None;
    }

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };

    if member.property.name.as_str() != "from" {
        return None;
    }

    if !matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Array") {
        return None;
    }

    let arg = call.arguments.first()?;
    let span = arg.span();
    Some(&source[span.start as usize..span.end as usize])
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferSetSize)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_spread_into_array_length() {
        let diags = lint("var n = [...mySet].length;");
        assert_eq!(diags.len(), 1, "[...mySet].length should be flagged");
    }

    #[test]
    fn test_flags_array_from_length() {
        let diags = lint("var n = Array.from(mySet).length;");
        assert_eq!(diags.len(), 1, "Array.from(mySet).length should be flagged");
    }

    #[test]
    fn test_allows_set_size() {
        let diags = lint("var n = mySet.size;");
        assert!(diags.is_empty(), "mySet.size should not be flagged");
    }

    #[test]
    fn test_allows_array_length() {
        let diags = lint("var n = myArray.length;");
        assert!(diags.is_empty(), "myArray.length should not be flagged");
    }

    #[test]
    fn test_allows_array_from_with_mapper() {
        let diags = lint("var n = Array.from(mySet, x => x * 2).length;");
        assert!(
            diags.is_empty(),
            "Array.from with mapper should not be flagged"
        );
    }

    #[test]
    fn test_allows_multi_element_spread() {
        let diags = lint("var n = [1, ...mySet].length;");
        assert!(
            diags.is_empty(),
            "array with multiple elements should not be flagged"
        );
    }
}
