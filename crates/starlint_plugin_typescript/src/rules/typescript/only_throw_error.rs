//! Rule: `typescript/only-throw-error`
//!
//! Disallow throwing non-Error values. Flags `throw` statements where the
//! argument is a literal (string, number, boolean, null, undefined) rather
//! than an Error object or variable that likely holds one.
//!
//! Simplified syntax-only version — full checking requires type information.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/only-throw-error";

/// Flags `throw` statements that throw literal values instead of Error objects.
#[derive(Debug)]
pub struct OnlyThrowError;

impl LintRule for OnlyThrowError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow throwing non-Error values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ThrowStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ThrowStatement(throw) = node else {
            return;
        };

        if is_non_error_literal(throw.argument, ctx) {
            // Fix: throw "msg" → throw new Error("msg")
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let arg_node = ctx.node(throw.argument);
                let arg_span = match arg_node {
                    Some(n) => n.span(),
                    None => return,
                };
                let is_string_or_template = matches!(
                    arg_node,
                    Some(AstNode::StringLiteral(_) | AstNode::TemplateLiteral(_))
                );
                source
                    .get(arg_span.start as usize..arg_span.end as usize)
                    .map(|text| {
                        let wrapped = if is_string_or_template {
                            format!("new Error({text})")
                        } else {
                            format!("new Error(String({text}))")
                        };
                        Fix {
                            kind: FixKind::SuggestionFix,
                            message: format!("Replace with `throw {wrapped}`"),
                            edits: vec![Edit {
                                span: Span::new(arg_span.start, arg_span.end),
                                replacement: wrapped,
                            }],
                            is_snippet: false,
                        }
                    })
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Expected an Error object to be thrown — do not throw literals".to_owned(),
                span: Span::new(throw.span.start, throw.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

/// Returns `true` if the expression (by `NodeId`) is a literal value that is not an Error:
/// string, number, boolean, null, undefined, bigint.
fn is_non_error_literal(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.node(id),
        Some(
            AstNode::StringLiteral(_)
                | AstNode::NumericLiteral(_)
                | AstNode::BooleanLiteral(_)
                | AstNode::NullLiteral(_)
                | AstNode::TemplateLiteral(_)
        )
    ) || is_undefined_identifier(id, ctx)
}

/// Returns `true` if the expression (by `NodeId`) is the identifier `undefined`.
fn is_undefined_identifier(id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.node(id), Some(AstNode::IdentifierReference(ident)) if ident.name == "undefined")
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(OnlyThrowError)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_throw_string() {
        let diags = lint("throw \"error message\";");
        assert_eq!(
            diags.len(),
            1,
            "throwing a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_number() {
        let diags = lint("throw 42;");
        assert_eq!(
            diags.len(),
            1,
            "throwing a number literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_boolean() {
        let diags = lint("throw true;");
        assert_eq!(
            diags.len(),
            1,
            "throwing a boolean literal should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_null() {
        let diags = lint("throw null;");
        assert_eq!(diags.len(), 1, "throwing null should be flagged");
    }

    #[test]
    fn test_flags_throw_undefined() {
        let diags = lint("throw undefined;");
        assert_eq!(diags.len(), 1, "throwing undefined should be flagged");
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('something went wrong');");
        assert!(diags.is_empty(), "throwing new Error should not be flagged");
    }

    #[test]
    fn test_allows_throw_variable() {
        let diags = lint("throw err;");
        assert!(
            diags.is_empty(),
            "throwing a variable should not be flagged"
        );
    }
}
