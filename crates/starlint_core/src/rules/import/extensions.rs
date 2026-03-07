//! Rule: `import/extensions`
//!
//! Ensure consistent use of file extension within import paths.
//! By default, this rule warns when an import path includes a file extension,
//! since bundlers and Node.js module resolution typically handle extensions
//! automatically.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Known JS/TS file extensions that should typically be omitted.
const JS_EXTENSIONS: &[&str] = &[".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts"];

/// Flags import paths that include file extensions.
#[derive(Debug)]
pub struct Extensions;

impl LintRule for Extensions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/extensions".to_owned(),
            description: "Ensure consistent use of file extension in import path".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        let source_value = import.source.as_str();

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
                let src_start = usize::try_from(import.source_span.start).unwrap_or(0);
                let src_end = usize::try_from(import.source_span.end).unwrap_or(0);
                let raw = source.get(src_start..src_end).unwrap_or("");
                // Remove the extension from just before the closing quote
                let ext_len = ext.len();
                let fix = raw.len().checked_sub(ext_len.saturating_add(1)).map(|cut| {
                    let mut fixed = String::with_capacity(raw.len().saturating_sub(ext_len));
                    fixed.push_str(raw.get(..cut).unwrap_or(""));
                    // Append closing quote
                    fixed.push_str(raw.get(raw.len().saturating_sub(1)..).unwrap_or(""));
                    Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Remove '{ext}' extension"),
                        edits: vec![Edit {
                            span: Span::new(import.source_span.start, import.source_span.end),
                            replacement: fixed,
                        }],
                        is_snippet: false,
                    }
                });

                ctx.report(Diagnostic {
                    rule_name: "import/extensions".to_owned(),
                    message: format!("Unexpected use of file extension '{ext}' in import path"),
                    span: Span::new(import.source_span.start, import.source_span.end),
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
    use super::*;
    use crate::lint_rule::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(Extensions)];
        lint_source(source, "test.js", &rules)
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
