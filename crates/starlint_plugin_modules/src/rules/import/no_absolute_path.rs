//! Rule: `import/no-absolute-path`
//!
//! Disallow absolute filesystem paths in import declarations. Absolute paths
//! are not portable across machines and break when the project is moved.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags import declarations that use absolute filesystem paths.
#[derive(Debug)]
pub struct NoAbsolutePath;

impl LintRule for NoAbsolutePath {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-absolute-path".to_owned(),
            description: "Disallow absolute paths in import declarations".to_owned(),
            category: Category::Suggestion,
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

        // Check for Unix absolute paths (/) and Windows absolute paths (C:\, D:\, etc.)
        let is_absolute = source_value.starts_with('/')
            || source_value.as_bytes().get(1).is_some_and(|b| *b == b':');

        if is_absolute {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove absolute path import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-absolute-path".to_owned(),
                message: format!("Do not use absolute path '{source_value}' in import"),
                span: Span::new(import.source_span.start, import.source_span.end),
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
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoAbsolutePath);

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
