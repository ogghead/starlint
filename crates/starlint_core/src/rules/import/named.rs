//! Rule: `import/named`
//!
//! Validate that named imports correspond to named exports in the resolved
//! module. Without full module resolution this rule performs a heuristic
//! check: it flags named imports from relative paths ending in `.json`
//! since JSON modules only expose a default export.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclarationSpecifier;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags named imports that are unlikely to exist in the resolved module.
#[derive(Debug)]
pub struct NamedExport;

impl NativeRule for NamedExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/named".to_owned(),
            description:
                "Validate that named imports correspond to named exports in the resolved module"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        // Type-only imports don't need runtime exports
        if import.import_kind.is_type() {
            return;
        }

        let source_value = import.source.value.as_str();

        // Heuristic: JSON modules only have a default export
        if !std::path::Path::new(source_value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            return;
        }

        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for spec in specifiers {
            if let ImportDeclarationSpecifier::ImportSpecifier(named) = spec {
                ctx.report_error(
                    "import/named",
                    &format!(
                        "'{}' is not exported from '{}' (JSON modules only have a default export)",
                        named.local.name.as_str(),
                        source_value,
                    ),
                    Span::new(named.span.start, named.span.end),
                );
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NamedExport)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_named_import_from_json() {
        let diags = lint(r#"import { foo } from "./data.json";"#);
        assert_eq!(
            diags.len(),
            1,
            "named import from JSON module should be flagged"
        );
    }

    #[test]
    fn test_allows_default_import_from_json() {
        let diags = lint(r#"import data from "./data.json";"#);
        assert!(
            diags.is_empty(),
            "default import from JSON should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_named_import_from_js() {
        let diags = lint(r#"import { foo } from "./module";"#);
        assert!(
            diags.is_empty(),
            "named import from JS module should not be flagged"
        );
    }
}
