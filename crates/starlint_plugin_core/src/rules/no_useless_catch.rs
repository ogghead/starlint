//! Rule: `no-useless-catch`
//!
//! Disallow catch clauses that only rethrow the caught error.
//! `try { ... } catch (e) { throw e; }` is equivalent to just the try body.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags catch clauses that only rethrow without transformation.
#[derive(Debug)]
pub struct NoUselessCatch;

impl LintRule for NoUselessCatch {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-catch".to_owned(),
            description: "Disallow catch clauses that only rethrow".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TryStatement])
    }

    #[allow(clippy::indexing_slicing)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TryStatement(stmt) = node else {
            return;
        };

        let Some(handler_id) = stmt.handler else {
            return;
        };

        let Some(AstNode::CatchClause(handler)) = ctx.node(handler_id) else {
            return;
        };

        // Must have a simple identifier parameter.
        let Some(param_id) = handler.param else {
            return;
        };

        let Some(AstNode::BindingIdentifier(param_ident)) = ctx.node(param_id) else {
            return;
        };
        let param_name = param_ident.name.clone();

        let handler_span = handler.span;
        let handler_body_id = handler.body;

        let Some(AstNode::BlockStatement(handler_body)) = ctx.node(handler_body_id) else {
            return;
        };

        // Body must have exactly one statement: `throw <same identifier>`.
        if handler_body.body.len() != 1 {
            return;
        }

        let first_stmt_id = handler_body.body[0];
        let Some(AstNode::ThrowStatement(throw_stmt)) = ctx.node(first_stmt_id) else {
            return;
        };

        let throw_arg_id = throw_stmt.argument;
        let Some(AstNode::IdentifierReference(thrown_id)) = ctx.node(throw_arg_id) else {
            return;
        };
        if thrown_id.name != param_name {
            return;
        }

        // This is a useless catch. Build the fix.
        let source = ctx.source_text();

        // Get the block span
        let block_span = ctx.node(stmt.block).map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );

        if stmt.finalizer.is_some() {
            // Has finally -- remove the catch clause, keep try + finally.
            let catch_end = handler_span.end;
            ctx.report(Diagnostic {
                rule_name: "no-useless-catch".to_owned(),
                message: "Unnecessary catch clause — only rethrows the error".to_owned(),
                span: Span::new(handler_span.start, handler_span.end),
                severity: Severity::Warning,
                help: Some("Remove the useless catch clause".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Remove catch clause".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(block_span.end, catch_end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        } else {
            // No finally -- unwrap the try block entirely.
            let block_start = usize::try_from(block_span.start).unwrap_or(0);
            let block_end = usize::try_from(block_span.end).unwrap_or(0);
            let Some(block_text) = source.get(block_start..block_end) else {
                return;
            };

            // Strip the outer braces from the try block: `{ body }` -> `body`.
            let inner = block_text
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .unwrap_or(block_text)
                .trim();

            let replacement = inner.to_owned();
            let fix_message = "Unwrap try block".to_owned();

            ctx.report(Diagnostic {
                rule_name: "no-useless-catch".to_owned(),
                message: "Unnecessary try/catch — catch clause only rethrows the error".to_owned(),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Warning,
                help: Some("Remove the try/catch wrapper".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: fix_message,
                    edits: vec![Edit {
                        span: Span::new(stmt.span.start, stmt.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUselessCatch);

    #[test]
    fn test_flags_useless_catch() {
        let diags = lint("try { doSomething(); } catch (e) { throw e; }");
        assert_eq!(diags.len(), 1, "should flag useless catch");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("doSomething();"),
            "fix should unwrap the try block"
        );
    }

    #[test]
    fn test_flags_useless_catch_with_finally() {
        let diags = lint("try { x(); } catch (e) { throw e; } finally { cleanup(); }");
        assert_eq!(
            diags.len(),
            1,
            "should flag useless catch when finally exists"
        );
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        // Fix removes the catch clause, keeping try + finally.
        assert!(fix.is_some(), "should provide a fix");
    }

    #[test]
    fn test_ignores_catch_with_transformation() {
        let diags = lint("try { x(); } catch (e) { throw new Error(e.message); }");
        assert!(
            diags.is_empty(),
            "catch with error transformation should not be flagged"
        );
    }

    #[test]
    fn test_ignores_catch_with_multiple_statements() {
        let diags = lint("try { x(); } catch (e) { console.log(e); throw e; }");
        assert!(
            diags.is_empty(),
            "catch with multiple statements should not be flagged"
        );
    }

    #[test]
    fn test_ignores_catch_throwing_different_error() {
        let diags = lint("try { x(); } catch (e) { throw otherError; }");
        assert!(
            diags.is_empty(),
            "catch throwing different identifier should not be flagged"
        );
    }

    #[test]
    fn test_ignores_catch_without_param() {
        let diags = lint("try { x(); } catch { throw new Error(); }");
        assert!(
            diags.is_empty(),
            "catch without param should not be flagged"
        );
    }

    #[test]
    fn test_ignores_destructured_catch_param() {
        let diags = lint("try { x(); } catch ({ message }) { throw message; }");
        assert!(
            diags.is_empty(),
            "destructured catch param should not be flagged"
        );
    }
}
