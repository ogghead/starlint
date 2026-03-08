//! Rule: `jest/no-mocks-import`
//!
//! Error when importing from a `__mocks__` directory.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::FixBuilder;
use starlint_rule_framework::fix_utils;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-mocks-import";

/// Flags imports from `__mocks__` directories.
#[derive(Debug)]
pub struct NoMocksImport;

impl LintRule for NoMocksImport {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow importing from `__mocks__` directories".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        if source_value.contains("__mocks__") {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove `__mocks__` import", FixKind::SuggestionFix)
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
    use super::*;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoMocksImport)];
        lint_source(source, "test.js", &rules)
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
