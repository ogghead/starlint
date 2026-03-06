//! Rule: `typescript/no-base-to-string`
//!
//! Disallow calling `.toString()` on object types that don't have a useful
//! `toString()` implementation. Calling `.toString()` on a plain object
//! returns `"[object Object]"` and on an array literal may produce
//! unexpected comma-separated output.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This AST-based heuristic flags `.toString()` calls where the receiver is
//! an object literal or array literal expression.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-base-to-string";

/// Flags `.toString()` calls on object literals and array literals which
/// produce unhelpful string representations.
#[derive(Debug)]
pub struct NoBaseToString;

impl NativeRule for NoBaseToString {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow calling `.toString()` on objects without a useful toString"
                .to_owned(),
            category: Category::Correctness,
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

        // We're looking for `<expr>.toString()` with no arguments
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "toString" {
            return;
        }

        // Only flag when called with zero arguments (standard toString)
        if !call.arguments.is_empty() {
            return;
        }

        // Check if the object (receiver) is an object literal or array literal,
        // unwrapping any parenthesized expressions first.
        let receiver = unwrap_parens(&member.object);
        let receiver_kind = match receiver {
            Expression::ObjectExpression(_) => Some("object literal"),
            Expression::ArrayExpression(_) => Some("array literal"),
            _ => None,
        };

        let Some(kind_name) = receiver_kind else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: format!(
                "Calling `.toString()` on an {kind_name} returns a useless default string representation"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Unwrap parenthesized expressions to get the inner expression.
///
/// Handles nested parentheses like `((expr))`.
fn unwrap_parens<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut current = expr;
    while let Expression::ParenthesizedExpression(paren) = current {
        current = &paren.expression;
    }
    current
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoBaseToString)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_literal_to_string() {
        let diags = lint("const s = ({}).toString();");
        assert_eq!(
            diags.len(),
            1,
            "calling toString() on an object literal should be flagged"
        );
    }

    #[test]
    fn test_flags_array_literal_to_string() {
        let diags = lint("const s = [1, 2, 3].toString();");
        assert_eq!(
            diags.len(),
            1,
            "calling toString() on an array literal should be flagged"
        );
    }

    #[test]
    fn test_allows_variable_to_string() {
        let diags = lint("const x = 42; const s = x.toString();");
        assert!(
            diags.is_empty(),
            "calling toString() on a variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_literal_to_string() {
        let diags = lint(r#"const s = "hello".toString();"#);
        assert!(
            diags.is_empty(),
            "calling toString() on a string literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_to_string_with_radix() {
        let diags = lint("const s = (255).toString(16);");
        assert!(
            diags.is_empty(),
            "calling toString() with a radix argument should not be flagged"
        );
    }
}
