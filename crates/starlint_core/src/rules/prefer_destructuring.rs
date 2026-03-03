//! Rule: `prefer-destructuring`
//!
//! Require destructuring from arrays and objects when accessing a specific
//! element or property directly. For example, prefer `const { x } = obj`
//! over `const x = obj.x`.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, VariableDeclarationKind};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations that could use destructuring.
#[derive(Debug)]
pub struct PreferDestructuring;

impl NativeRule for PreferDestructuring {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-destructuring".to_owned(),
            description: "Prefer destructuring from arrays and objects".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclaration(decl) = kind else {
            return;
        };

        // Only check const/let declarations (not var — legacy code patterns)
        if decl.kind == VariableDeclarationKind::Var {
            return;
        }

        for declarator in &decl.declarations {
            // Must be a simple identifier binding (not already destructured)
            // If it has exactly one binding identifier, it's a simple binding.
            // Destructuring patterns would have zero (or multiple) binding identifiers
            // at the direct pattern level, or no match here.
            let bindings = declarator.id.get_binding_identifiers();
            if bindings.len() != 1 {
                continue;
            }

            let Some(init) = &declarator.init else {
                continue;
            };

            // Check if init is a member expression like `obj.prop` or `arr[0]`
            match init {
                Expression::StaticMemberExpression(member) => {
                    // obj.prop — suggest { prop } = obj
                    let binding_name = declarator
                        .id
                        .get_binding_identifiers()
                        .first()
                        .map(|b| b.name.as_str());
                    let prop_name = member.property.name.as_str();

                    // Only suggest if the variable name matches the property name
                    if binding_name == Some(prop_name) {
                        ctx.report_warning(
                            "prefer-destructuring",
                            &format!("Use object destructuring: `{{ {prop_name} }} = ...`"),
                            Span::new(declarator.span.start, declarator.span.end),
                        );
                    }
                }
                Expression::ComputedMemberExpression(member) => {
                    // arr[0] — suggest destructuring for numeric indices
                    if let Expression::NumericLiteral(_) = &member.expression {
                        ctx.report_warning(
                            "prefer-destructuring",
                            "Use array destructuring instead of indexed access",
                            Span::new(declarator.span.start, declarator.span.end),
                        );
                    }
                }
                _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDestructuring)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_property_access() {
        let diags = lint("const x = obj.x;");
        assert_eq!(
            diags.len(),
            1,
            "same-name property access should be flagged"
        );
    }

    #[test]
    fn test_allows_different_name() {
        // const y = obj.x — names differ, not a simple destructuring target
        let diags = lint("const y = obj.x;");
        assert!(diags.is_empty(), "different name should not be flagged");
    }

    #[test]
    fn test_allows_already_destructured() {
        let diags = lint("const { x } = obj;");
        assert!(
            diags.is_empty(),
            "already destructured should not be flagged"
        );
    }

    #[test]
    fn test_allows_var() {
        let diags = lint("var x = obj.x;");
        assert!(diags.is_empty(), "var should not be checked");
    }

    #[test]
    fn test_flags_array_index() {
        let diags = lint("const x = arr[0];");
        assert_eq!(diags.len(), 1, "indexed access should be flagged");
    }

    #[test]
    fn test_allows_computed_non_numeric() {
        let diags = lint("const x = obj[key];");
        assert!(
            diags.is_empty(),
            "computed non-numeric should not be flagged"
        );
    }
}
