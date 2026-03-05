//! Rule: `no-const-enum` (oxc)
//!
//! Flag TypeScript `const enum` declarations. `const enum` has compatibility
//! issues and doesn't work well with `--isolatedModules`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `const enum` declarations in TypeScript.
#[derive(Debug)]
pub struct NoConstEnum;

impl NativeRule for NoConstEnum {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-const-enum".to_owned(),
            description: "Disallow TypeScript `const enum` declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSEnumDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSEnumDeclaration(decl) = kind else {
            return;
        };

        if decl.r#const {
            // Fix: remove "const " prefix — the enum keyword starts 6 bytes after the const keyword
            let fix = Some(Fix {
                message: "Remove `const` keyword".to_owned(),
                edits: vec![Edit {
                    span: Span::new(decl.span.start, decl.span.start.saturating_add(6)),
                    replacement: String::new(),
                }],
            });

            ctx.report(Diagnostic {
                rule_name: "no-const-enum".to_owned(),
                message: "Do not use `const enum`. Use a regular `enum` or a union type instead"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: Some("Remove the `const` keyword".to_owned()),
                fix,
                labels: vec![],
            });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstEnum)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_const_enum() {
        let diags = lint("const enum Color { Red, Blue }");
        assert_eq!(diags.len(), 1, "const enum should be flagged");
    }

    #[test]
    fn test_allows_regular_enum() {
        let diags = lint("enum Color { Red, Blue }");
        assert!(diags.is_empty(), "regular enum should not be flagged");
    }

    #[test]
    fn test_allows_non_enum() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "non-enum code should not be flagged");
    }
}
