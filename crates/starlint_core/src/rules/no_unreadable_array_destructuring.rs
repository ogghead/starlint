//! Rule: `no-unreadable-array-destructuring` (unicorn)
//!
//! Disallow array destructuring with more than 3 consecutive ignored elements.
//! For example, `const [,,,,val] = arr` is hard to read.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Maximum consecutive holes allowed before flagging.
const MAX_CONSECUTIVE_HOLES: usize = 3;

/// Flags unreadable array destructuring with many consecutive holes.
#[derive(Debug)]
pub struct NoUnreadableArrayDestructuring;

impl LintRule for NoUnreadableArrayDestructuring {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unreadable-array-destructuring".to_owned(),
            description: "Disallow array destructuring with many consecutive ignored elements"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclarator])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclarator(decl) = node else {
            return;
        };

        let Some(AstNode::ArrayPattern(array_pat)) = ctx.node(decl.id) else {
            return;
        };

        // Count consecutive None elements (holes)
        let mut consecutive_holes = 0_usize;
        let mut max_holes = 0_usize;
        for element in &*array_pat.elements {
            if element.is_none() {
                consecutive_holes = consecutive_holes.saturating_add(1);
                if consecutive_holes > max_holes {
                    max_holes = consecutive_holes;
                }
            } else {
                consecutive_holes = 0;
            }
        }

        if max_holes > MAX_CONSECUTIVE_HOLES {
            ctx.report(Diagnostic {
                rule_name: "no-unreadable-array-destructuring".to_owned(),
                message:
                    "Array destructuring with many consecutive ignored elements is hard to read — \
                 use `array[index]` instead"
                        .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
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
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnreadableArrayDestructuring)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_many_holes() {
        let diags = lint("const [,,,,val] = arr;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_few_holes() {
        let diags = lint("const [,,val] = arr;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_normal_destructuring() {
        let diags = lint("const [a, b, c] = arr;");
        assert!(diags.is_empty());
    }
}
