//! Rule: `typescript/no-var-requires`
//!
//! Disallow `require()` in variable declarations. In `TypeScript` projects,
//! `require()` calls bypass the type system. Prefer `import` declarations
//! which are statically analyzed and type-checked.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags variable declarations whose initializer is a `require()` call.
#[derive(Debug)]
pub struct NoVarRequires;

impl LintRule for NoVarRequires {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-var-requires".to_owned(),
            description: "Disallow `require()` in variable declarations".to_owned(),
            category: Category::Suggestion,
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

        let Some(init_id) = decl.init else {
            return;
        };

        if is_require_call(init_id, ctx) {
            // Fix: const x = require("foo") -> import x from "foo" (simple identifier case)
            #[allow(clippy::as_conversions)]
            let fix = if let Some(AstNode::BindingIdentifier(id)) = ctx.node(decl.id) {
                if let Some(AstNode::CallExpression(call)) = ctx.node(init_id) {
                    call.arguments.first().and_then(|&arg_id| {
                        if let Some(AstNode::StringLiteral(lit)) = ctx.node(arg_id) {
                            let name = id.name.as_str();
                            let module = lit.value.as_str();
                            let replacement = format!("import {name} from \"{module}\"");
                            Some(Fix {
                                kind: FixKind::SuggestionFix,
                                message: format!("Replace with `{replacement}`"),
                                edits: vec![Edit {
                                    span: Span::new(decl.span.start, decl.span.end),
                                    replacement,
                                }],
                                is_snippet: false,
                            })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: "typescript/no-var-requires".to_owned(),
                message: "Use `import` instead of `require()` in variable declarations".to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if an expression is a call to `require`.
fn is_require_call(expr_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::CallExpression(call)) = ctx.node(expr_id) else {
        return false;
    };

    matches!(ctx.node(call.callee), Some(AstNode::IdentifierReference(ident)) if ident.name.as_str() == "require")
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoVarRequires)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_const_require() {
        let diags = lint("const x = require(\"foo\");");
        assert_eq!(diags.len(), 1, "`const x = require(...)` should be flagged");
    }

    #[test]
    fn test_flags_let_require() {
        let diags = lint("let x = require(\"bar\");");
        assert_eq!(diags.len(), 1, "`let x = require(...)` should be flagged");
    }

    #[test]
    fn test_allows_import() {
        let diags = lint("import x from \"foo\";");
        assert!(
            diags.is_empty(),
            "`import` declaration should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_require_call() {
        let diags = lint("const x = foo();");
        assert!(
            diags.is_empty(),
            "non-`require` call in variable init should not be flagged"
        );
    }

    #[test]
    fn test_allows_variable_without_init() {
        let diags = lint("let x;");
        assert!(
            diags.is_empty(),
            "variable without initializer should not be flagged"
        );
    }
}
