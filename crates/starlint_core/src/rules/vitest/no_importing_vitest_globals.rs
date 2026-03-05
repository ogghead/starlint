//! Rule: `vitest/no-importing-vitest-globals`
//!
//! Warn when Vitest globals (`describe`, `it`, `test`, `expect`, `vi`,
//! `beforeEach`, `afterEach`, `beforeAll`, `afterAll`) are explicitly imported
//! from the `vitest` package. When `globals: true` is set in Vitest config,
//! these are available without import.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclarationSpecifier;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vitest/no-importing-vitest-globals";

/// Global names that Vitest provides when `globals: true`.
const VITEST_GLOBALS: &[&str] = &[
    "describe",
    "it",
    "test",
    "expect",
    "vi",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
    "suite",
    "bench",
];

/// Warn when Vitest globals are explicitly imported from `vitest`.
#[derive(Debug)]
pub struct NoImportingVitestGlobals;

impl NativeRule for NoImportingVitestGlobals {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Disallow explicit imports of Vitest globals when `globals: true` is configured"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        if source_value != "vitest" {
            return;
        }

        // Check if any imported specifier is a known Vitest global.
        let Some(specifiers) = &import.specifiers else {
            return;
        };

        for specifier in specifiers {
            if let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier {
                let imported_name = named.imported.name().as_str();
                if VITEST_GLOBALS.contains(&imported_name) {
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!(
                            "Do not import `{imported_name}` from `vitest` — it is available as a global when `globals: true` is configured"
                        ),
                        span: Span::new(named.span.start, named.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoImportingVitestGlobals)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_describe_import() {
        let source = r#"import { describe, it, expect } from "vitest";"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 3, "all three Vitest globals should be flagged");
    }

    #[test]
    fn test_allows_non_global_imports() {
        let source = r#"import { expectTypeOf } from "vitest";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-global Vitest imports should not be flagged"
        );
    }

    #[test]
    fn test_allows_imports_from_other_packages() {
        let source = r#"import { describe } from "mocha";"#;
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "imports from other packages should not be flagged"
        );
    }
}
