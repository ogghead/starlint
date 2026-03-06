//! Rule: `import/extensions`
//!
//! Ensure consistent use of file extension within import paths.
//! By default, this rule warns when an import path includes a file extension,
//! since bundlers and Node.js module resolution typically handle extensions
//! automatically.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
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
            fix_kind: FixKind::SafeFix,
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

        // Skip bare specifiers (not relative or absolute paths)
        if !source_value.starts_with('.') && !source_value.starts_with('/') {
            return;
        }

        for ext in JS_EXTENSIONS {
            if source_value.ends_with(ext) {
                // Strip the extension from the source value.
                // The span includes quotes, so we compute the replacement
                // by removing the extension from the raw source text.
                let source = ctx.source_text();
                let src_start = usize::try_from(import.source.span.start).unwrap_or(0);
                let src_end = usize::try_from(import.source.span.end).unwrap_or(0);
                let raw = source.get(src_start..src_end).unwrap_or("");
                // Remove the extension from just before the closing quote
                let ext_len = ext.len();
                let fix = raw.len().checked_sub(ext_len.saturating_add(1)).map(|cut| {
                    let mut fixed = String::with_capacity(raw.len().saturating_sub(ext_len));
                    fixed.push_str(raw.get(..cut).unwrap_or(""));
                    // Append closing quote
                    fixed.push_str(raw.get(raw.len().saturating_sub(1)..).unwrap_or(""));
                    Fix {
                        message: format!("Remove '{ext}' extension"),
                        edits: vec![Edit {
                            span: Span::new(import.source.span.start, import.source.span.end),
                            replacement: fixed,
                        }],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: "import/extensions".to_owned(),
                    message: format!("Unexpected use of file extension '{ext}' in import path"),
                    span: Span::new(import.source.span.start, import.source.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix,
                    labels: vec![],
                });
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
