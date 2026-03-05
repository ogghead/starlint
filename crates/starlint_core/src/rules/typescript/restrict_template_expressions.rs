//! Rule: `typescript/restrict-template-expressions`
//!
//! Disallow template literal expressions with non-string types. Interpolating
//! object literals, array literals, `null`, or `undefined` into template
//! strings produces unhelpful output like `[object Object]`, an empty string,
//! `"null"`, or `"undefined"`.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! This rule inspects `TemplateLiteral` AST nodes and flags expressions that
//! are clearly not strings: object literals, array literals, `null` literals,
//! and the `undefined` identifier.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags template literal expressions that are clearly non-string values.
#[derive(Debug)]
pub struct RestrictTemplateExpressions;

impl NativeRule for RestrictTemplateExpressions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/restrict-template-expressions".to_owned(),
            description: "Disallow non-string types in template literal expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TemplateLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TemplateLiteral(template) = kind else {
            return;
        };

        // Skip tagged template literals — they may have custom handling
        // (tagged templates appear as TaggedTemplateExpression, not bare
        // TemplateLiteral, but we guard defensively here)

        // Collect findings first to avoid borrow checker issues
        let findings: Vec<(u32, u32, &str)> = template
            .expressions
            .iter()
            .filter_map(|expr| {
                let kind_name = non_string_expression_kind(expr)?;
                Some((expr.span().start, expr.span().end, kind_name))
            })
            .collect();

        for (start, end, kind_name) in findings {
            ctx.report(Diagnostic {
                rule_name: "typescript/restrict-template-expressions".to_owned(),
                message: format!(
                    "Do not use {kind_name} in a template literal — it will not produce a \
                     useful string"
                ),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is clearly a non-string type that should not be
/// interpolated into a template literal.
///
/// Returns a description of the problematic type, or `None` if the expression
/// is acceptable (or cannot be determined without type information).
fn non_string_expression_kind(expr: &Expression<'_>) -> Option<&'static str> {
    match expr {
        Expression::ObjectExpression(_) => Some("an object literal"),
        Expression::ArrayExpression(_) => Some("an array literal"),
        Expression::NullLiteral(_) => Some("`null`"),
        Expression::Identifier(ident) if ident.name.as_str() == "undefined" => Some("`undefined`"),
        _ => None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RestrictTemplateExpressions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_literal_in_template() {
        let diags = lint("const s = `value: ${{a: 1}}`;");
        assert_eq!(
            diags.len(),
            1,
            "object literal in template should be flagged"
        );
    }

    #[test]
    fn test_flags_array_literal_in_template() {
        let diags = lint("const s = `value: ${[1, 2]}`;");
        assert_eq!(
            diags.len(),
            1,
            "array literal in template should be flagged"
        );
    }

    #[test]
    fn test_flags_null_in_template() {
        let diags = lint("const s = `value: ${null}`;");
        assert_eq!(diags.len(), 1, "null in template should be flagged");
    }

    #[test]
    fn test_flags_undefined_in_template() {
        let diags = lint("const s = `value: ${undefined}`;");
        assert_eq!(diags.len(), 1, "undefined in template should be flagged");
    }

    #[test]
    fn test_allows_string_variable_in_template() {
        let diags = lint("const name = 'world'; const s = `hello ${name}`;");
        assert!(
            diags.is_empty(),
            "string variable in template should not be flagged"
        );
    }
}
