//! Rule: `import/no-relative-parent-imports`
//!
//! Forbid importing from parent directories (`../`). Parent imports can
//! create tightly-coupled code and make refactoring harder.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags imports whose source begins with `../`.
#[derive(Debug)]
pub struct NoRelativeParentImports;

impl NativeRule for NoRelativeParentImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-relative-parent-imports".to_owned(),
            description: "Forbid importing from parent directories".to_owned(),
            category: Category::Style,
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

        let source_value = import.source.value.as_str();

        if source_value.starts_with("../") || source_value == ".." {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove parent directory import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-relative-parent-imports".to_owned(),
                message: "Relative parent imports are not allowed".to_owned(),
                span: import_span,
                severity: Severity::Warning,
                help: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRelativeParentImports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_parent_import() {
        let diags = lint(r#"import foo from "../utils";"#);
        assert_eq!(diags.len(), 1, "parent directory import should be flagged");
    }

    #[test]
    fn test_flags_deep_parent_import() {
        let diags = lint(r#"import bar from "../../lib/helpers";"#);
        assert_eq!(
            diags.len(),
            1,
            "deep parent directory import should be flagged"
        );
    }

    #[test]
    fn test_allows_sibling_import() {
        let diags = lint(r#"import baz from "./sibling";"#);
        assert!(
            diags.is_empty(),
            "sibling directory import should not be flagged"
        );
    }
}
