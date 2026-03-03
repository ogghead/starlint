//! Rule: `typescript/dot-notation`
//!
//! Enforce dot notation whenever possible instead of bracket notation for
//! property access. Writing `obj["property"]` is harder to read than
//! `obj.property` and should be avoided when the property name is a valid
//! JavaScript identifier.
//!
//! This rule uses the AST to find `ComputedMemberExpression` nodes whose
//! property is a `StringLiteral` containing a valid identifier name.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags bracket notation (`obj["prop"]`) when dot notation (`obj.prop`) works.
#[derive(Debug)]
pub struct DotNotation;

impl NativeRule for DotNotation {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/dot-notation".to_owned(),
            description: "Enforce dot notation over bracket notation for property access"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ComputedMemberExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ComputedMemberExpression(computed) = kind else {
            return;
        };

        let Expression::StringLiteral(lit) = &computed.expression else {
            return;
        };

        let property_name = lit.value.as_str();

        if is_valid_js_identifier(property_name) {
            ctx.report_warning(
                "typescript/dot-notation",
                &format!(
                    "Use dot notation `obj.{property_name}` instead of bracket notation `obj[\"{property_name}\"]`"
                ),
                Span::new(computed.span.start, computed.span.end),
            );
        }
    }
}

/// Check whether a string is a valid JavaScript identifier that can be used
/// with dot notation.
///
/// A valid identifier starts with a letter, `_`, or `$`, and contains only
/// letters, digits, `_`, or `$`. It must also not be empty.
///
/// This does not check for reserved words — JavaScript allows reserved words
/// as property names in dot notation (e.g. `obj.class` is valid).
fn is_valid_js_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character: must be a letter, `_`, or `$`
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    // Remaining characters: letters, digits, `_`, or `$`
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DotNotation)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_bracket_with_simple_string() {
        let diags = lint(r#"const x = obj["property"];"#);
        assert_eq!(diags.len(), 1, "`obj[\"property\"]` should be flagged");
    }

    #[test]
    fn test_flags_bracket_with_underscore_name() {
        let diags = lint(r#"const x = obj["_private"];"#);
        assert_eq!(diags.len(), 1, "`obj[\"_private\"]` should be flagged");
    }

    #[test]
    fn test_allows_bracket_with_hyphenated_name() {
        let diags = lint(r#"const x = obj["my-property"];"#);
        assert!(
            diags.is_empty(),
            "bracket notation with hyphens should not be flagged"
        );
    }

    #[test]
    fn test_allows_bracket_with_space_in_name() {
        let diags = lint(r#"const x = obj["has space"];"#);
        assert!(
            diags.is_empty(),
            "bracket notation with spaces should not be flagged"
        );
    }

    #[test]
    fn test_allows_bracket_with_numeric_key() {
        let diags = lint("const x = arr[0];");
        assert!(
            diags.is_empty(),
            "numeric bracket access should not be flagged"
        );
    }
}
