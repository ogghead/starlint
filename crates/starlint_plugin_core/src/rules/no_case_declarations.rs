//! Rule: `no-case-declarations`
//!
//! Disallow lexical declarations in case/default clauses. Lexical declarations
//! (`let`, `const`, `class`, `function`) in case clauses are visible to the
//! entire switch block but only initialized when the case is reached, which
//! can lead to unexpected errors.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::VariableDeclarationKind;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags lexical declarations (`let`, `const`, `class`, `function`) inside
/// `case` / `default` clauses that are not wrapped in a block.
#[derive(Debug)]
pub struct NoCaseDeclarations;

impl LintRule for NoCaseDeclarations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-case-declarations".to_owned(),
            description: "Disallow lexical declarations in case/default clauses".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::SwitchCase])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::SwitchCase(case) = node else {
            return;
        };

        for stmt_id in &*case.consequent {
            if let Some(span) = lexical_declaration_span(*stmt_id, ctx) {
                // Fix: wrap the entire case body in { }
                let fix = case
                    .consequent
                    .first()
                    .zip(case.consequent.last())
                    .and_then(|(first_id, last_id)| {
                        let first_span = ctx.node(*first_id).map(starlint_ast::AstNode::span)?;
                        let last_span = ctx.node(*last_id).map(starlint_ast::AstNode::span)?;
                        let body_start = first_span.start;
                        let body_end = last_span.end;
                        Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Wrap case body in a block `{ }`".to_owned(),
                            edits: vec![
                                Edit {
                                    span: Span::new(body_start, body_start),
                                    replacement: "{ ".to_owned(),
                                },
                                Edit {
                                    span: Span::new(body_end, body_end),
                                    replacement: " }".to_owned(),
                                },
                            ],
                            is_snippet: false,
                        })
                    });

                ctx.report(Diagnostic {
                    rule_name: "no-case-declarations".to_owned(),
                    message: "Unexpected lexical declaration in case clause".to_owned(),
                    span,
                    severity: Severity::Error,
                    help: None,
                    fix,
                    labels: vec![],
                });
            }
        }
    }
}

/// If the statement is a lexical declaration, return its span.
fn lexical_declaration_span(stmt_id: NodeId, ctx: &LintContext<'_>) -> Option<Span> {
    let stmt = ctx.node(stmt_id)?;
    match stmt {
        AstNode::VariableDeclaration(decl)
            if decl.kind == VariableDeclarationKind::Let
                || decl.kind == VariableDeclarationKind::Const =>
        {
            Some(Span::new(decl.span.start, decl.span.end))
        }
        AstNode::Function(func) => Some(Span::new(func.span.start, func.span.end)),
        AstNode::Class(class) => Some(Span::new(class.span.start, class.span.end)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoCaseDeclarations)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_let_in_case() {
        let diags = lint("switch (x) { case 0: let y = 1; break; }");
        assert_eq!(diags.len(), 1, "let in case should be flagged");
    }

    #[test]
    fn test_flags_const_in_case() {
        let diags = lint("switch (x) { case 0: const y = 1; break; }");
        assert_eq!(diags.len(), 1, "const in case should be flagged");
    }

    #[test]
    fn test_flags_function_in_case() {
        let diags = lint("switch (x) { case 0: function f() {} break; }");
        assert_eq!(diags.len(), 1, "function decl in case should be flagged");
    }

    #[test]
    fn test_flags_class_in_case() {
        let diags = lint("switch (x) { case 0: class C {} break; }");
        assert_eq!(diags.len(), 1, "class decl in case should be flagged");
    }

    #[test]
    fn test_allows_var_in_case() {
        let diags = lint("switch (x) { case 0: var y = 1; break; }");
        assert!(diags.is_empty(), "var in case should not be flagged");
    }

    #[test]
    fn test_allows_let_in_block() {
        let diags = lint("switch (x) { case 0: { let y = 1; break; } }");
        assert!(
            diags.is_empty(),
            "let inside block in case should not be flagged"
        );
    }

    #[test]
    fn test_flags_in_default() {
        let diags = lint("switch (x) { default: let y = 1; }");
        assert_eq!(diags.len(), 1, "let in default should be flagged");
    }
}
