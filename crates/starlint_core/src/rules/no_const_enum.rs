//! Rule: `no-const-enum` (oxc)
//!
//! Flag TypeScript `const enum` declarations. `const enum` has compatibility
//! issues and doesn't work well with `--isolatedModules`.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
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
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSEnumDeclaration(decl) = kind else {
            return;
        };

        if decl.r#const {
            ctx.report_warning(
                "no-const-enum",
                "Do not use `const enum`. Use a regular `enum` or a union type instead",
                Span::new(decl.span.start, decl.span.end),
            );
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
