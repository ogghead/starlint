//! Rule: `node/no-exports-assign`
//!
//! Disallow direct assignment to `exports`. In `CommonJS`, the `exports`
//! variable is a reference to `module.exports`. Reassigning `exports`
//! directly (e.g. `exports = {}`) breaks that reference and does not
//! change what the module actually exports.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags direct assignment to the `exports` identifier.
///
/// `exports = value` breaks the module reference. Use
/// `module.exports = value` or `exports.prop = value` instead.
#[derive(Debug)]
pub struct NoExportsAssign;

impl LintRule for NoExportsAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-exports-assign".to_owned(),
            description: "Disallow direct assignment to `exports`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Only flag bare `exports = ...` (identifier target).
        // `exports.foo = bar` is fine (extending, not reassigning).
        let id_span = match ctx.node(assign.left) {
            Some(AstNode::IdentifierReference(id)) if id.name.as_str() == "exports" => {
                Span::new(id.span.start, id.span.end)
            }
            _ => return,
        };

        ctx.report(Diagnostic {
            rule_name: "node/no-exports-assign".to_owned(),
            message: "Direct assignment to `exports` breaks the module reference \u{2014} use `module.exports` or `exports.prop` instead".to_owned(),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Error,
            help: Some("Replace `exports` with `module.exports`".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Replace `exports` with `module.exports`".to_owned(),
                edits: vec![Edit {
                    span: id_span,
                    replacement: "module.exports".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExportsAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_exports_reassignment() {
        let diags = lint("exports = {};");
        assert_eq!(
            diags.len(),
            1,
            "direct assignment to exports should be flagged"
        );
    }

    #[test]
    fn test_flags_exports_assign_variable() {
        let diags = lint("exports = something;");
        assert_eq!(
            diags.len(),
            1,
            "assigning variable to exports should be flagged"
        );
    }

    #[test]
    fn test_allows_exports_property_assignment() {
        let diags = lint("exports.foo = bar;");
        assert!(diags.is_empty(), "exports.foo = bar should not be flagged");
    }

    #[test]
    fn test_allows_module_exports_assignment() {
        let diags = lint("module.exports = {};");
        assert!(
            diags.is_empty(),
            "module.exports assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_assignment() {
        let diags = lint("x = 1;");
        assert!(diags.is_empty(), "normal assignment should not be flagged");
    }
}
