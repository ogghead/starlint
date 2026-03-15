//! Rule: `import/no-namespace`
//!
//! Forbid namespace (wildcard `*`) imports. Namespace imports import the
//! entire module which defeats tree-shaking and makes it harder to
//! identify which exports are actually used.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags namespace (wildcard `* as`) imports.
#[derive(Debug)]
pub struct NoNamespace;

impl LintRule for NoNamespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-namespace".to_owned(),
            description: "Forbid namespace (wildcard `*`) imports".to_owned(),
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

        // Check if any specifier is a namespace import (import * as X).
        // In starlint_ast, specifiers resolve to ImportSpecifier nodes.
        // A namespace import has is_namespace: true or local name starts from `* as`.
        // We detect namespace imports by checking source text for `* as`.
        let source = ctx.source_text();
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        let import_text = source
            .get(import.span.start as usize..import.span.end as usize)
            .unwrap_or("");
        let has_namespace = import_text.contains("* as");

        if has_namespace {
            ctx.report(Diagnostic {
                rule_name: "import/no-namespace".to_owned(),
                message: "Unexpected namespace import — use named imports instead".to_owned(),
                span: Span::new(import.span.start, import.span.end),
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

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoNamespace);

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
