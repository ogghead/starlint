//! Rule: `no-debugger` (unified `LintRule` version)
//!
//! Disallow `debugger` statements. These should never appear in production code.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::lint_rule::{LintContext, LintRule};

/// Flags `debugger` statements and offers a safe fix to remove them.
#[derive(Debug)]
pub struct NoDebugger;

impl LintRule for NoDebugger {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-debugger".to_owned(),
            description: "Disallow `debugger` statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::DebuggerStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        if let AstNode::DebuggerStatement(stmt) = node {
            let span = Span::new(stmt.span.start, stmt.span.end);
            ctx.report(Diagnostic {
                rule_name: "no-debugger".to_owned(),
                message: "Unexpected `debugger` statement".to_owned(),
                span,
                severity: Severity::Error,
                help: Some("Remove the `debugger` statement before deploying".to_owned()),
                fix: FixBuilder::new("Remove `debugger` statement", FixKind::SafeFix)
                    .delete(span)
                    .build(),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::*;
    use crate::ast_converter;
    use crate::lint_rule::LintRule;
    use crate::traversal::{LintDispatchTable, traverse_ast_tree};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        let tree = ast_converter::convert(&parsed.program);
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDebugger)];
        let table = LintDispatchTable::build_from_indices(&rules, &[0]);
        traverse_ast_tree(
            &tree,
            &rules,
            &table,
            &[],
            source,
            Path::new("test.js"),
            None,
        )
    }

    #[test]
    fn flags_debugger() {
        let diags = lint("debugger;\nconst x = 1;");
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("no-debugger")
        );
        assert!(diags.first().is_some_and(|d| d.fix.is_some()));
    }

    #[test]
    fn clean_file() {
        let diags = lint("const x = 1;\nexport default x;");
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_debugger() {
        let diags = lint("debugger;\nconst x = 1;\ndebugger;");
        assert_eq!(diags.len(), 2);
    }
}
