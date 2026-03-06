//! Rule: `typescript/prefer-enum-initializers`
//!
//! Require explicit initializers for all enum members. When enum members rely
//! on implicit auto-incrementing values, inserting or removing a member can
//! silently change the values of subsequent members, leading to subtle bugs
//! (e.g. serialized values no longer matching, switch cases breaking).
//! Requiring explicit initializers makes the intent clear and prevents
//! accidental value drift.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags enum members that lack an explicit initializer.
#[derive(Debug)]
pub struct PreferEnumInitializers;

impl NativeRule for PreferEnumInitializers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-enum-initializers".to_owned(),
            description: "Require explicit initializers for enum members".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSEnumDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSEnumDeclaration(decl) = kind else {
            return;
        };

        let mut index: u32 = 0;
        for member in &decl.body.members {
            if member.initializer.is_none() {
                let member_name = member.id.static_name();

                // Insert ` = <index>` right after the member identifier (at end of member span)
                let fix = Some(Fix {
                    message: format!("Add initializer `= {index}`"),
                    edits: vec![Edit {
                        span: Span::new(member.span.end, member.span.end),
                        replacement: format!(" = {index}"),
                    }],
                    is_snippet: false,
                });

                ctx.report(Diagnostic {
                    rule_name: "typescript/prefer-enum-initializers".to_owned(),
                    message: format!(
                        "Enum member `{member_name}` should have an explicit initializer"
                    ),
                    span: Span::new(member.span.start, member.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Add `= {index}` to `{member_name}`")),
                    fix,
                    labels: vec![],
                });
            }
            index = index.saturating_add(1);
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

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferEnumInitializers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_member_without_initializer() {
        let diags = lint("enum Direction { Up, Down }");
        assert_eq!(
            diags.len(),
            2,
            "both enum members without initializers should be flagged"
        );
    }

    #[test]
    fn test_flags_mixed_members() {
        let diags = lint("enum E { A = 0, B, C = 2 }");
        assert_eq!(
            diags.len(),
            1,
            "only the member without an initializer should be flagged"
        );
    }

    #[test]
    fn test_allows_all_initialized_numeric() {
        let diags = lint("enum Direction { Up = 0, Down = 1, Left = 2, Right = 3 }");
        assert!(
            diags.is_empty(),
            "enum with all numeric initializers should not be flagged"
        );
    }

    #[test]
    fn test_allows_all_initialized_string() {
        let diags = lint(r#"enum Color { Red = "RED", Green = "GREEN" }"#);
        assert!(
            diags.is_empty(),
            "enum with all string initializers should not be flagged"
        );
    }

    #[test]
    fn test_flags_single_uninitialized_member() {
        let diags = lint("enum E { A }");
        assert_eq!(
            diags.len(),
            1,
            "single enum member without initializer should be flagged"
        );
    }
}
