//! Rule: `no-var` (unified `LintRule` version)
//!
//! Disallow `var` declarations. Prefer `let` and `const`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `var` declarations, suggesting `let` instead.
#[derive(Debug)]
pub struct NoVar;

impl LintRule for NoVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-var".to_owned(),
            description: "Require `let` or `const` instead of `var`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if let AstNode::VariableDeclaration(decl) = node {
            if decl.kind == VariableDeclarationKind::Var {
                let var_span = Span::new(decl.span.start, decl.span.start.saturating_add(3));

                ctx.report(Diagnostic {
                    rule_name: "no-var".to_owned(),
                    message: "Unexpected `var`, use `let` or `const` instead".to_owned(),
                    span: Span::new(decl.span.start, decl.span.end),
                    severity: Severity::Warning,
                    help: Some(
                        "Replace `var` with `let` (or `const` if never reassigned)".to_owned(),
                    ),
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Replace `var` with `let`".to_owned(),
                        edits: vec![Edit {
                            span: var_span,
                            replacement: "let".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use starlint_parser::ParseOptions;

    use super::*;
    use crate::lint_rule::LintRule;
    use crate::traversal::{LintDispatchTable, traverse_ast_tree};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let path = Path::new("test.js");
        let tree = starlint_parser::parse(source, ParseOptions::from_path(path)).tree;
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoVar)];
        let table = LintDispatchTable::build_from_indices(&rules, &[0]);
        traverse_ast_tree(&tree, &rules, &table, &[], source, path, None)
    }

    #[test]
    fn flags_var() {
        let diags = lint("var x = 1;");
        assert_eq!(diags.len(), 1);
        assert!(diags.first().is_some_and(|d| d.fix.is_some()));
    }

    #[test]
    fn allows_let() {
        assert!(lint("let x = 1;").is_empty());
    }

    #[test]
    fn allows_const() {
        assert!(lint("const x = 1;").is_empty());
    }

    #[test]
    fn fix_replaces_var() {
        let diags = lint("var x = 1;");
        let edit = diags
            .first()
            .and_then(|d| d.fix.as_ref())
            .and_then(|f| f.edits.first());
        assert_eq!(edit.map(|e| e.replacement.as_str()), Some("let"));
    }
}
