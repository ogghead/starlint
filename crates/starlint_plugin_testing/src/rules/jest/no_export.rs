//! Rule: `jest/no-export`
//!
//! Error when test files contain exports. Test files should not export anything.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-export";

/// Flags export declarations in test files.
#[derive(Debug)]
pub struct NoExport;

/// Check if a file path looks like a test file.
fn is_test_file(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains(".test.") || path_str.contains(".spec.")
}

impl LintRule for NoExport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow exports from test files".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ExportAllDeclaration,
            AstNodeType::ExportDefaultDeclaration,
            AstNodeType::ExportNamedDeclaration,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Only apply to test files
        if !is_test_file(ctx.file_path()) {
            return;
        }

        let span = match node {
            AstNode::ExportNamedDeclaration(decl) => Span::new(decl.span.start, decl.span.end),
            AstNode::ExportDefaultDeclaration(decl) => Span::new(decl.span.start, decl.span.end),
            AstNode::ExportAllDeclaration(decl) => Span::new(decl.span.start, decl.span.end),
            _ => return,
        };

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message: "Test files should not export anything".to_owned(),
            span,
            severity: Severity::Error,
            help: Some("Remove the export from this test file".to_owned()),
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExport)];
        lint_source(source, "test.test.ts", &rules)
    }

    fn lint_non_test(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExport)];
        lint_source(source, "utils.ts", &rules)
    }

    #[test]
    fn test_flags_named_export_in_test() {
        let diags = lint("export const helper = () => {};");
        assert_eq!(
            diags.len(),
            1,
            "named export in test file should be flagged"
        );
    }

    #[test]
    fn test_flags_default_export_in_test() {
        let diags = lint("export default function() {}");
        assert_eq!(
            diags.len(),
            1,
            "default export in test file should be flagged"
        );
    }

    #[test]
    fn test_allows_export_in_non_test() {
        let diags = lint_non_test("export const helper = () => {};");
        assert!(
            diags.is_empty(),
            "export in non-test file should not be flagged"
        );
    }
}
