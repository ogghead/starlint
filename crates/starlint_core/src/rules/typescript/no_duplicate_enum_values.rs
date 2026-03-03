//! Rule: `typescript/no-duplicate-enum-values`
//!
//! Disallow duplicate enum member values. When multiple enum members share the
//! same initializer value (string or number literal), the later members silently
//! shadow earlier ones, which is almost always a mistake.

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags enum declarations that contain members with duplicate initializer values.
#[derive(Debug)]
pub struct NoDuplicateEnumValues;

impl NativeRule for NoDuplicateEnumValues {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-duplicate-enum-values".to_owned(),
            description: "Disallow duplicate enum member values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSEnumDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSEnumDeclaration(decl) = kind else {
            return;
        };

        let mut seen = HashSet::new();

        for member in &decl.body.members {
            let Some(ref init) = member.initializer else {
                // Auto-incremented members — no explicit value to check.
                continue;
            };

            let Some(value_key) = static_initializer_key(init) else {
                continue;
            };

            if !seen.insert(value_key.clone()) {
                ctx.report_error(
                    "typescript/no-duplicate-enum-values",
                    &format!("Duplicate enum value `{value_key}`"),
                    Span::new(member.span.start, member.span.end),
                );
            }
        }
    }
}

/// Extract a comparable string key from an enum member initializer expression.
///
/// Returns `Some` for string and numeric literals, `None` for anything else
/// (computed expressions, identifiers, etc.).
fn static_initializer_key(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::StringLiteral(lit) => Some(format!("\"{}\"", lit.value)),
        Expression::NumericLiteral(lit) => Some(lit.value.to_string()),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDuplicateEnumValues)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_number_values() {
        let diags = lint("enum E { A = 1, B = 1 }");
        assert_eq!(
            diags.len(),
            1,
            "duplicate number enum values should be flagged"
        );
    }

    #[test]
    fn test_flags_duplicate_string_values() {
        let diags = lint(r#"enum E { A = "x", B = "x" }"#);
        assert_eq!(
            diags.len(),
            1,
            "duplicate string enum values should be flagged"
        );
    }

    #[test]
    fn test_allows_unique_values() {
        let diags = lint("enum E { A = 1, B = 2 }");
        assert!(diags.is_empty(), "unique enum values should not be flagged");
    }

    #[test]
    fn test_allows_auto_incremented() {
        let diags = lint("enum E { A, B }");
        assert!(
            diags.is_empty(),
            "auto-incremented enum members should not be flagged"
        );
    }
}
