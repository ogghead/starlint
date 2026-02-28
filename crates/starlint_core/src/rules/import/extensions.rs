//! Rule: `import/extensions`
//!
//! Ensure consistent use of file extension within import paths.
//! By default, this rule warns when an import path includes a file extension,
//! since bundlers and Node.js module resolution typically handle extensions
//! automatically.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Known JS/TS file extensions that should typically be omitted.
const JS_EXTENSIONS: &[&str] = &[".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts"];

/// Flags import paths that include file extensions.
#[derive(Debug)]
pub struct Extensions;

impl NativeRule for Extensions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/extensions".to_owned(),
            description: "Ensure consistent use of file extension in import path".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source_value = import.source.value.as_str();

        // Skip bare specifiers (not relative or absolute paths)
        if !source_value.starts_with('.') && !source_value.starts_with('/') {
            return;
        }

        for ext in JS_EXTENSIONS {
            if source_value.ends_with(ext) {
                ctx.report_warning(
                    "import/extensions",
                    &format!("Unexpected use of file extension '{ext}' in import path"),
                    Span::new(import.source.span.start, import.source.span.end),
                );
                return;
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(Extensions)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_js_extension() {
        let diags = lint(r#"import foo from "./module.js";"#);
        assert_eq!(
            diags.len(),
            1,
            "import with .js extension should be flagged"
        );
    }

    #[test]
    fn test_allows_no_extension() {
        let diags = lint(r#"import foo from "./module";"#);
        assert!(
            diags.is_empty(),
            "import without extension should not be flagged"
        );
    }

    #[test]
    fn test_allows_bare_specifier_with_extension() {
        let diags = lint(r#"import foo from "some-pkg/file.js";"#);
        assert!(
            diags.is_empty(),
            "bare specifier with extension should not be flagged"
        );
    }
}
