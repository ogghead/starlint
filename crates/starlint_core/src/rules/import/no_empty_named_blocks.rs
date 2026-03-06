//! Rule: `import/no-empty-named-blocks`
//!
//! Forbid empty named import blocks (`import {} from 'mod'`).
//! An empty import block is likely a mistake or leftover from refactoring.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags import declarations with empty named import blocks.
#[derive(Debug)]
pub struct NoEmptyNamedBlocks;

impl NativeRule for NoEmptyNamedBlocks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-empty-named-blocks".to_owned(),
            description: "Forbid empty named import blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        // Side-effect imports (`import 'mod'`) have no specifiers — that's valid
        let Some(specifiers) = &import.specifiers else {
            return;
        };

        // Empty named block: the specifiers list exists but is empty
        // This catches `import {} from 'mod'`
        if specifiers.is_empty() {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove the empty import statement", FixKind::SafeFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-empty-named-blocks".to_owned(),
                message: "Unexpected empty named import block".to_owned(),
                span: import_span,
                severity: Severity::Warning,
                help: Some("Remove the empty import statement".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyNamedBlocks)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_named_block() {
        let diags = lint(r#"import {} from "mod";"#);
        assert_eq!(diags.len(), 1, "empty named import block should be flagged");
    }

    #[test]
    fn test_allows_named_imports() {
        let diags = lint(r#"import { foo } from "mod";"#);
        assert!(
            diags.is_empty(),
            "non-empty named import block should not be flagged"
        );
    }

    #[test]
    fn test_allows_side_effect_import() {
        let diags = lint(r#"import "mod";"#);
        assert!(diags.is_empty(), "side-effect import should not be flagged");
    }
}
