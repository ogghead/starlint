//! Rule: `no-undef` (eslint)
//!
//! Disallow the use of undeclared variables. This helps catch typos
//! and missing imports/declarations.
//!
//! Note: This is a simplified version that checks for unresolved references
//! in the semantic model. Well-known globals (console, setTimeout, etc.)
//! are allowed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags references to undeclared variables.
#[derive(Debug)]
pub struct NoUndef;

/// Well-known browser/Node.js globals that should not be flagged.
const KNOWN_GLOBALS: &[&str] = &[
    "undefined",
    "NaN",
    "Infinity",
    "globalThis",
    "console",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "setImmediate",
    "clearImmediate",
    "queueMicrotask",
    "requestAnimationFrame",
    "cancelAnimationFrame",
    "fetch",
    "URL",
    "URLSearchParams",
    "TextEncoder",
    "TextDecoder",
    "AbortController",
    "AbortSignal",
    "Blob",
    "File",
    "FormData",
    "Headers",
    "Request",
    "Response",
    "Event",
    "EventTarget",
    "CustomEvent",
    "WebSocket",
    "Worker",
    "SharedWorker",
    "MessageChannel",
    "MessagePort",
    "BroadcastChannel",
    "structuredClone",
    "atob",
    "btoa",
    "crypto",
    "performance",
    "navigator",
    "location",
    "history",
    "document",
    "window",
    "self",
    "global",
    "process",
    "Buffer",
    "require",
    "module",
    "exports",
    "__dirname",
    "__filename",
    // Built-in constructors and objects
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
    "FinalizationRegistry",
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
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
    // Built-in functions
    "eval",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "escape",
    "unescape",
    // Test globals
    "describe",
    "it",
    "test",
    "expect",
    "beforeAll",
    "afterAll",
    "beforeEach",
    "afterEach",
    "jest",
    "vi",
    // DOM
    "alert",
    "confirm",
    "prompt",
    "HTMLElement",
    "Element",
    "Node",
    "NodeList",
    "DocumentFragment",
];

impl LintRule for NoUndef {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-undef".to_owned(),
            description: "Disallow the use of undeclared variables".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::IdentifierReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::IdentifierReference(ident) = node else {
            return;
        };

        // Skip known globals
        if KNOWN_GLOBALS.contains(&ident.name.as_str()) {
            return;
        }

        // If the reference is resolved (bound to a symbol), it's fine.
        if ctx.is_reference_resolved_at(ident.name.as_str(), ident.span) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-undef".to_owned(),
            message: format!("'{}' is not defined", ident.name),
            span: Span::new(ident.span.start, ident.span.end),
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUndef)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_declared_variable() {
        let diags = lint("const x = 1; foo(x);");
        // `foo` is undeclared but let's just check x isn't flagged
        assert!(
            !diags.iter().any(|d| d.message.contains("'x'")),
            "declared variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_known_globals() {
        let diags = lint("console.log('hello');");
        assert!(diags.is_empty(), "console should not be flagged");
    }

    #[test]
    fn test_allows_math() {
        let diags = lint("Math.floor(1.5);");
        assert!(diags.is_empty(), "Math should not be flagged");
    }

    #[test]
    fn test_allows_json() {
        let diags = lint("JSON.parse('{}');");
        assert!(diags.is_empty(), "JSON should not be flagged");
    }
}
