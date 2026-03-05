//! Rule: `import/no-absolute-path`
//!
//! Disallow absolute filesystem paths in import declarations. Absolute paths
//! are not portable across machines and break when the project is moved.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags import declarations that use absolute filesystem paths.
#[derive(Debug)]
pub struct NoAbsolutePath;

impl NativeRule for NoAbsolutePath {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-absolute-path".to_owned(),
            description: "Disallow absolute paths in import declarations".to_owned(),
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

        // Check for Unix absolute paths (/) and Windows absolute paths (C:\, D:\, etc.)
        let is_absolute = source_value.starts_with('/')
            || source_value.as_bytes().get(1).is_some_and(|b| *b == b':');

        if is_absolute {
            ctx.report(Diagnostic {
                rule_name: "import/no-absolute-path".to_owned(),
                message: format!("Do not use absolute path '{source_value}' in import"),
                span: Span::new(import.source.span.start, import.source.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAbsolutePath)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unix_absolute_path() {
        let diags = lint(r#"import foo from "/usr/local/lib/foo";"#);
        assert_eq!(diags.len(), 1, "Unix absolute path should be flagged");
    }

    #[test]
    fn test_allows_relative_path() {
        let diags = lint(r#"import foo from "./module";"#);
        assert!(diags.is_empty(), "relative path should not be flagged");
    }

    #[test]
    fn test_allows_bare_specifier() {
        let diags = lint(r#"import foo from "lodash";"#);
        assert!(diags.is_empty(), "bare specifier should not be flagged");
    }
}
