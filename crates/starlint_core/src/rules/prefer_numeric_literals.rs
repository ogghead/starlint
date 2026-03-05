//! Rule: `prefer-numeric-literals`
//!
//! Disallow `parseInt()` and `Number.parseInt()` for binary, octal, and hex
//! literals. Use `0b`, `0o`, and `0x` prefix notation instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `parseInt(str, radix)` where radix is 2, 8, or 16.
#[derive(Debug)]
pub struct PreferNumericLiterals;

impl NativeRule for PreferNumericLiterals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-numeric-literals".to_owned(),
            description: "Disallow `parseInt()` for binary, octal, and hex literals".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_parse_int = match &call.callee {
            Expression::Identifier(id) => id.name.as_str() == "parseInt",
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "parseInt"
                    && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "Number")
            }
            _ => false,
        };

        if !is_parse_int || call.arguments.len() < 2 {
            return;
        }

        // Check if the second argument is a literal 2, 8, or 16
        if let Some(Argument::NumericLiteral(num)) = call.arguments.get(1) {
            let radix = num.value;
            let prefix = if (radix - 2.0).abs() < f64::EPSILON {
                Some("0b")
            } else if (radix - 8.0).abs() < f64::EPSILON {
                Some("0o")
            } else if (radix - 16.0).abs() < f64::EPSILON {
                Some("0x")
            } else {
                None
            };

            if let Some(lit_prefix) = prefix {
                // Extract string value from first argument
                let fix = call.arguments.first().and_then(|arg| {
                    if let Argument::StringLiteral(s) = arg {
                        Some(Fix {
                            message: "Use numeric literal".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(call.span.start, call.span.end),
                                replacement: format!("{lit_prefix}{}", s.value.as_str()),
                            }],
                        })
                    } else {
                        None
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: "prefer-numeric-literals".to_owned(),
                    message: "Use a numeric literal instead of `parseInt()`".to_owned(),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Use `{lit_prefix}` literal notation")),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferNumericLiterals)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_hex_parse_int() {
        let diags = lint("parseInt('1A', 16);");
        assert_eq!(diags.len(), 1, "parseInt with radix 16 should be flagged");
    }

    #[test]
    fn test_flags_binary_parse_int() {
        let diags = lint("parseInt('111110111', 2);");
        assert_eq!(diags.len(), 1, "parseInt with radix 2 should be flagged");
    }

    #[test]
    fn test_flags_octal_parse_int() {
        let diags = lint("parseInt('767', 8);");
        assert_eq!(diags.len(), 1, "parseInt with radix 8 should be flagged");
    }

    #[test]
    fn test_allows_decimal_parse_int() {
        let diags = lint("parseInt('10', 10);");
        assert!(
            diags.is_empty(),
            "parseInt with radix 10 should not be flagged"
        );
    }
}
