//! Rule: `jest/no-mocks-import`
//!
//! Error when importing from a `__mocks__` directory.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-mocks-import";

/// Flags imports from `__mocks__` directories.
#[derive(Debug)]
pub struct NoMocksImport;

impl NativeRule for NoMocksImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow importing from `__mocks__` directories".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
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

        if source_value.contains("__mocks__") {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove `__mocks__` import")
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message:
                    "Do not import from `__mocks__` — mocks are automatically resolved by Jest"
                        .to_owned(),
                span: import_span,
                severity: Severity::Error,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMocksImport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mocks_import() {
        let diags = lint("import foo from './__mocks__/foo';");
        assert_eq!(diags.len(), 1, "import from `__mocks__` should be flagged");
    }

    #[test]
    fn test_flags_nested_mocks_import() {
        let diags = lint("import { bar } from '../__mocks__/utils/bar';");
        assert_eq!(
            diags.len(),
            1,
            "import from nested `__mocks__` path should be flagged"
        );
    }

    #[test]
    fn test_allows_regular_import() {
        let diags = lint("import foo from './foo';");
        assert!(diags.is_empty(), "regular imports should not be flagged");
    }
}
