//! Rule: `no-global-assign` (eslint)
//!
//! Disallow assignment to native/global objects. Assigning to built-in
//! globals like `Object`, `Array`, `undefined`, etc. can cause unexpected
//! behavior throughout the application.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags assignment to native/global objects.
#[derive(Debug)]
pub struct NoGlobalAssign;

/// Built-in globals that should never be reassigned.
const READ_ONLY_GLOBALS: &[&str] = &[
    "undefined",
    "NaN",
    "Infinity",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Symbol",
    "BigInt",
    "Function",
    "Date",
    "RegExp",
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
    "AggregateError",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "WeakRef",
    "Promise",
    "Proxy",
    "Reflect",
    "JSON",
    "Math",
    "Intl",
    "ArrayBuffer",
    "SharedArrayBuffer",
    "DataView",
    "Atomics",
    "globalThis",
    "eval",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
];

impl LintRule for NoGlobalAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-global-assign".to_owned(),
            description: "Disallow assignment to native/global objects".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Extract owned data from the identifier to avoid borrow conflicts
        // with ctx.report().
        let (name, id_span) = {
            let Some(AstNode::IdentifierReference(id)) = ctx.node(assign.left) else {
                return;
            };
            (id.name.clone(), id.span)
        };

        if !READ_ONLY_GLOBALS.contains(&name.as_str()) {
            return;
        }

        // Only flag if the name is not locally declared (unresolved reference).
        if ctx.is_reference_resolved_at(&name, id_span) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-global-assign".to_owned(),
            message: format!("Do not assign to the global variable '{name}'"),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Error,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoGlobalAssign)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_object_reassignment() {
        let diags = lint("Object = null;");
        assert_eq!(diags.len(), 1, "reassigning Object should be flagged");
    }

    #[test]
    fn test_flags_undefined_reassignment() {
        let diags = lint("undefined = true;");
        assert_eq!(diags.len(), 1, "reassigning undefined should be flagged");
    }

    #[test]
    fn test_allows_local_variable() {
        let diags = lint("let Object = {}; Object = null;");
        assert!(
            diags.is_empty(),
            "reassigning local variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_assignment() {
        let diags = lint("let x = 1; x = 2;");
        assert!(diags.is_empty(), "regular assignment should not be flagged");
    }
}
