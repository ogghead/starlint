//! Rule: `prefer-at` (unicorn)
//!
//! Prefer `.at()` for index access from the end of an array/string.
//! `array.at(-1)` is more readable than `array[array.length - 1]`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `arr[arr.length - 1]` patterns that should use `.at(-1)`.
#[derive(Debug)]
pub struct PreferAt;

impl NativeRule for PreferAt {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-at".to_owned(),
            description: "Prefer `.at()` for index access from the end".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ComputedMemberExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ComputedMemberExpression(computed) = kind else {
            return;
        };

        // Check for `obj[obj.length - N]` pattern
        let Expression::BinaryExpression(bin) = &computed.expression else {
            return;
        };

        if !matches!(bin.operator, oxc_ast::ast::BinaryOperator::Subtraction) {
            return;
        }

        // Left side should be `something.length`
        let Expression::StaticMemberExpression(member) = &bin.left else {
            return;
        };

        if member.property.name != "length" {
            return;
        }

        // Right side should be a numeric literal
        let Expression::NumericLiteral(num_lit) = &bin.right else {
            return;
        };

        // The object being accessed and the `.length` owner should be the same
        if let (Expression::Identifier(obj_id), Expression::Identifier(len_obj_id)) =
            (&computed.object, &member.object)
        {
            if obj_id.name == len_obj_id.name {
                let obj_name = obj_id.name.as_str();
                // Format the negative index value
                let n = num_lit.value;
                #[allow(clippy::cast_possible_truncation)]
                let neg_index = if (n - n.round()).abs() < f64::EPSILON {
                    format!("-{}", n as i64)
                } else {
                    format!("-{n}")
                };
                let replacement = format!("{obj_name}.at({neg_index})");

                ctx.report(Diagnostic {
                    rule_name: "prefer-at".to_owned(),
                    message: "Prefer `.at()` for index access from the end of an array or string"
                        .to_owned(),
                    span: Span::new(computed.span.start, computed.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace with `{replacement}`")),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(computed.span.start, computed.span.end),
                            replacement,
                        }],
                        is_snippet: false,
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferAt)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_length_minus_one() {
        let diags = lint("var x = arr[arr.length - 1];");
        assert_eq!(diags.len(), 1, "arr[arr.length - 1] should be flagged");
    }

    #[test]
    fn test_flags_length_minus_two() {
        let diags = lint("var x = arr[arr.length - 2];");
        assert_eq!(diags.len(), 1, "arr[arr.length - 2] should be flagged");
    }

    #[test]
    fn test_allows_at() {
        let diags = lint("var x = arr.at(-1);");
        assert!(diags.is_empty(), ".at(-1) should not be flagged");
    }

    #[test]
    fn test_allows_normal_index() {
        let diags = lint("var x = arr[0];");
        assert!(diags.is_empty(), "arr[0] should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("var x = arr[other.length - 1];");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }
}
