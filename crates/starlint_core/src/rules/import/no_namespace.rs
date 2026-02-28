//! Rule: `import/no-namespace`
//!
//! Forbid namespace (wildcard `*`) imports. Namespace imports import the
//! entire module which defeats tree-shaking and makes it harder to
//! identify which exports are actually used.

use oxc_ast::AstKind;
use oxc_ast::ast::ImportDeclarationSpecifier;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags namespace (wildcard `* as`) imports.
#[derive(Debug)]
pub struct NoNamespace;

impl NativeRule for NoNamespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-namespace".to_owned(),
            description: "Forbid namespace (wildcard `*`) imports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let Some(specifiers) = &import.specifiers else {
            return;
        };
        let has_namespace = specifiers.iter().any(|spec| {
            matches!(
                spec,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
            )
        });

        if has_namespace {
            ctx.report_warning(
                "import/no-namespace",
                "Unexpected namespace import — use named imports instead",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamespace)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_namespace_import() {
        let diags = lint(r#"import * as utils from "utils";"#);
        assert_eq!(diags.len(), 1, "namespace import should be flagged");
    }

    #[test]
    fn test_allows_named_import() {
        let diags = lint(r#"import { foo, bar } from "utils";"#);
        assert!(diags.is_empty(), "named import should not be flagged");
    }

    #[test]
    fn test_allows_default_import() {
        let diags = lint(r#"import utils from "utils";"#);
        assert!(diags.is_empty(), "default import should not be flagged");
    }
}
