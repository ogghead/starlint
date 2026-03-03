//! Rule: `import/no-unassigned-import`
//!
//! Forbid unassigned (side-effect) imports like `import 'polyfill'`.
//! Side-effect imports make it hard to determine what a module depends on
//! and can cause unexpected behavior.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags side-effect imports that have no specifiers.
#[derive(Debug)]
pub struct NoUnassignedImport;

impl NativeRule for NoUnassignedImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-unassigned-import".to_owned(),
            description: "Forbid unassigned (side-effect) imports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        // Side-effect import: `import 'foo'` — specifiers is None
        // Empty named block: `import {} from 'foo'` — specifiers is Some([])
        let is_unassigned = import
            .specifiers
            .as_ref()
            .is_none_or(|specs| specs.is_empty());

        if is_unassigned {
            ctx.report_warning(
                "import/no-unassigned-import",
                "Unexpected side-effect import with no bindings",
                Span::new(import.span.start, import.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnassignedImport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_side_effect_import() {
        let diags = lint(r#"import "polyfill";"#);
        assert_eq!(diags.len(), 1, "side-effect import should be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo } from "module";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_default_import() {
        let diags = lint(r#"import foo from "module";"#);
        assert!(diags.is_empty(), "default import should not be flagged");
    }
}
