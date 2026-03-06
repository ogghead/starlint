//! Rule: `no-const-enum` (oxc)
//!
//! Flag TypeScript `const enum` declarations. `const enum` has compatibility
//! issues and doesn't work well with `--isolatedModules`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags `const enum` declarations in TypeScript.
#[derive(Debug)]
pub struct NoConstEnum;

impl LintRule for NoConstEnum {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-const-enum".to_owned(),
            description: "Disallow TypeScript `const enum` declarations".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSEnumDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSEnumDeclaration(decl) = node else {
            return;
        };

        if decl.is_const {
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove `const` keyword".to_owned(),
                edits: vec![Edit {
                    span: Span::new(decl.span.start, decl.span.start.saturating_add(6)),
                    replacement: String::new(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "no-const-enum".to_owned(),
                message: "Do not use `const enum`. Use a regular `enum` or a union type instead"
                    .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: Some("Remove the `const` keyword".to_owned()),
                fix,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConstEnum)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_const_enum() {
        let diags = lint("const enum Color { Red, Blue }");
        assert_eq!(diags.len(), 1, "const enum should be flagged");
    }

    #[test]
    fn test_allows_regular_enum() {
        let diags = lint("enum Color { Red, Blue }");
        assert!(diags.is_empty(), "regular enum should not be flagged");
    }

    #[test]
    fn test_allows_non_enum() {
        let diags = lint("const x = 1;");
        assert!(diags.is_empty(), "non-enum code should not be flagged");
    }
}
