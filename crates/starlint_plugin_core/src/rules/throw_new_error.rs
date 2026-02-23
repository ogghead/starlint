//! Rule: `throw-new-error`
//!
//! Require `new` when throwing Error constructors. `throw Error("msg")` works
//! but is inconsistent — `throw new Error("msg")` is the standard form.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Standard JavaScript error constructors.
const ERROR_CONSTRUCTORS: &[&str] = &[
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
    "AggregateError",
];

/// Flags `throw Error(...)` expressions that are missing `new`.
#[derive(Debug)]
pub struct ThrowNewError;

impl LintRule for ThrowNewError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "throw-new-error".to_owned(),
            description: "Require `new` when throwing Error constructors".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ThrowStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ThrowStatement(stmt) = node else {
            return;
        };

        // Must be a direct call: `throw Error(...)`, not `throw new Error(...)`.
        let (name, callee_start) = {
            let Some(AstNode::CallExpression(call)) = ctx.node(stmt.argument) else {
                return;
            };
            let callee_id = call.callee;
            let Some(AstNode::IdentifierReference(id)) = ctx.node(callee_id) else {
                return;
            };
            (id.name.clone(), id.span.start)
        };
        if !ERROR_CONSTRUCTORS.contains(&name.as_str()) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "throw-new-error".to_owned(),
            message: format!("Use `new {name}()` instead of `{name}()`"),
            span: Span::new(stmt.span.start, stmt.span.end),
            severity: Severity::Error,
            help: Some(format!("Add `new` before `{name}`")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Add `new` before `{name}`"),
                edits: vec![Edit {
                    span: Span::new(callee_start, callee_start),
                    replacement: "new ".to_owned(),
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
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ThrowNewError)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_throw_error() {
        let diags = lint("throw Error('msg');");
        assert_eq!(diags.len(), 1, "should flag throw Error()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("new "),
            "fix should insert 'new '"
        );
    }

    #[test]
    fn test_flags_throw_type_error() {
        let diags = lint("throw TypeError('msg');");
        assert_eq!(diags.len(), 1, "should flag throw TypeError()");
    }

    #[test]
    fn test_flags_throw_range_error() {
        let diags = lint("throw RangeError('msg');");
        assert_eq!(diags.len(), 1, "should flag throw RangeError()");
    }

    #[test]
    fn test_allows_throw_new_error() {
        let diags = lint("throw new Error('msg');");
        assert!(diags.is_empty(), "throw new Error() should not be flagged");
    }

    #[test]
    fn test_allows_throw_variable() {
        let diags = lint("throw err;");
        assert!(diags.is_empty(), "throw variable should not be flagged");
    }

    #[test]
    fn test_allows_throw_string() {
        let diags = lint("throw 'error';");
        assert!(
            diags.is_empty(),
            "throw string literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_error_call() {
        let diags = lint("throw myFunction('msg');");
        assert!(
            diags.is_empty(),
            "non-error function call should not be flagged"
        );
    }
}
