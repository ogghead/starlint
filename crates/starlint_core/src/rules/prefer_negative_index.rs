//! Rule: `prefer-negative-index` (unicorn)
//!
//! Prefer negative index over `.length - index` for methods that support it.
//! Methods like `.slice()`, `.at()`, `.splice()` accept negative indices.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.slice(arr.length - N)` and similar patterns.
#[derive(Debug)]
pub struct PreferNegativeIndex;

/// Methods that accept negative indices.
const NEGATIVE_INDEX_METHODS: &[&str] = &["slice", "splice", "at", "with"];

impl NativeRule for PreferNegativeIndex {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-negative-index".to_owned(),
            description: "Prefer negative index over .length - index".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `something.method(something.length - N)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();
        if !NEGATIVE_INDEX_METHODS.contains(&method_name) {
            return;
        }

        // Check arguments for `.length - N` pattern
        for arg in &call.arguments {
            let oxc_ast::ast::Argument::BinaryExpression(bin) = arg else {
                continue;
            };

            if !matches!(bin.operator, oxc_ast::ast::BinaryOperator::Subtraction) {
                continue;
            }

            // Left side should be `something.length`
            let Expression::StaticMemberExpression(len_member) = &bin.left else {
                continue;
            };

            if len_member.property.name != "length" {
                continue;
            }

            // Right side should be a numeric literal
            let Expression::NumericLiteral(num_lit) = &bin.right else {
                continue;
            };

            // Check that the object and .length owner are the same identifier
            if let (Expression::Identifier(obj_id), Expression::Identifier(len_obj_id)) =
                (&member.object, &len_member.object)
            {
                if obj_id.name == len_obj_id.name {
                    let n = num_lit.value;
                    #[allow(clippy::cast_possible_truncation)]
                    let neg_val = if (n - n.round()).abs() < f64::EPSILON {
                        format!("-{}", n as i64)
                    } else {
                        format!("-{n}")
                    };

                    ctx.report(Diagnostic {
                        rule_name: "prefer-negative-index".to_owned(),
                        message: format!(
                            "Use a negative index instead of `.length` subtraction in `.{method_name}()`"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Warning,
                        help: Some(format!("Replace with `{neg_val}`")),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Replace `{obj_id}.length - {n}` with `{neg_val}`", obj_id = obj_id.name),
                            edits: vec![Edit {
                                span: Span::new(bin.span.start, bin.span.end),
                                replacement: neg_val,
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                    return;
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferNegativeIndex)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_slice_length_minus() {
        let diags = lint("arr.slice(arr.length - 2);");
        assert_eq!(
            diags.len(),
            1,
            "arr.slice(arr.length - 2) should be flagged"
        );
    }

    #[test]
    fn test_allows_slice_negative() {
        let diags = lint("arr.slice(-2);");
        assert!(diags.is_empty(), "arr.slice(-2) should not be flagged");
    }

    #[test]
    fn test_allows_different_objects() {
        let diags = lint("arr.slice(other.length - 2);");
        assert!(diags.is_empty(), "different objects should not be flagged");
    }

    #[test]
    fn test_allows_non_negative_index_method() {
        let diags = lint("arr.push(arr.length - 1);");
        assert!(
            diags.is_empty(),
            "non-negative-index method should not be flagged"
        );
    }
}
