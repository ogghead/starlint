//! Rule: `typescript/prefer-literal-enum-member`
//!
//! Prefer literal values in enum members rather than computed values. Enum
//! members with computed initializers (identifiers, call expressions, binary
//! expressions, etc.) make the enum harder to reason about at a glance and can
//! introduce unexpected runtime behavior. Literal values (strings, numbers,
//! unary negation of a number) are always safe and clear.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags enum members whose initializers are not literal values.
#[derive(Debug)]
pub struct PreferLiteralEnumMember;

impl NativeRule for PreferLiteralEnumMember {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-literal-enum-member".to_owned(),
            description: "Prefer literal values in enum members".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSEnumMember])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSEnumMember(member) = kind else {
            return;
        };

        let Some(ref init) = member.initializer else {
            // No initializer — auto-incremented; this is fine.
            return;
        };

        if is_literal_value(init) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/prefer-literal-enum-member".to_owned(),
            message: "Enum member should be initialized with a literal value".to_owned(),
            span: Span::new(member.span.start, member.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check whether an expression is a valid literal enum initializer.
///
/// Accepted forms:
/// - String literal (`"hello"`)
/// - Numeric literal (`42`)
/// - Boolean literal (`true`, `false`)
/// - Template literal with no expressions (`` `hello` ``)
/// - Unary negation of a numeric literal (`-1`)
fn is_literal_value(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_) => true,
        Expression::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        Expression::UnaryExpression(unary) => {
            unary.operator == UnaryOperator::UnaryNegation
                && matches!(&unary.argument, Expression::NumericLiteral(_))
        }
        _ => false,
    }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferLiteralEnumMember)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_function_call_initializer() {
        let diags = lint("enum E { A = foo() }");
        assert_eq!(
            diags.len(),
            1,
            "enum member with function call initializer should be flagged"
        );
    }

    #[test]
    fn test_flags_identifier_initializer() {
        let diags = lint("enum E { A = x }");
        assert_eq!(
            diags.len(),
            1,
            "enum member with identifier initializer should be flagged"
        );
    }

    #[test]
    fn test_allows_numeric_literal() {
        let diags = lint("enum E { A = 1 }");
        assert!(
            diags.is_empty(),
            "enum member with numeric literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_literal() {
        let diags = lint("enum E { A = \"hello\" }");
        assert!(
            diags.is_empty(),
            "enum member with string literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_initializer() {
        let diags = lint("enum E { A }");
        assert!(
            diags.is_empty(),
            "enum member without initializer should not be flagged"
        );
    }

    #[test]
    fn test_allows_negative_number() {
        let diags = lint("enum E { A = -1 }");
        assert!(
            diags.is_empty(),
            "enum member with negative number literal should not be flagged"
        );
    }
}
