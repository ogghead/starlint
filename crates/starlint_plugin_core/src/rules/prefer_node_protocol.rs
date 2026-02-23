//! Rule: `prefer-node-protocol` (unicorn)
//!
//! Prefer using the `node:` protocol when importing Node.js built-in
//! modules. Using `node:fs` instead of `fs` makes it clear the import
//! is a built-in module and prevents conflicts with user packages.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Node.js built-in module names.
const NODE_BUILTINS: &[&str] = &[
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "test",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// Flags imports of Node.js built-ins without the `node:` protocol.
#[derive(Debug)]
pub struct PreferNodeProtocol;

impl LintRule for PreferNodeProtocol {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-node-protocol".to_owned(),
            description: "Prefer node: protocol for Node.js built-in modules".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        let source = import.source.as_str();

        // Already uses node: protocol
        if source.starts_with("node:") {
            return;
        }

        // Check for subpath imports like "fs/promises"
        let module_name = source.split('/').next().unwrap_or(source);

        if NODE_BUILTINS.contains(&module_name) {
            let source_span = Span::new(import.source_span.start, import.source_span.end);
            // The span includes the quotes, so replace the content inside the quotes.
            // source_span covers the full string literal including quotes, e.g. 'fs'.
            // We need to replace the inner content: start+1 to end-1.
            let inner_span = Span::new(
                source_span.start.saturating_add(1),
                source_span.end.saturating_sub(1),
            );
            ctx.report(Diagnostic {
                rule_name: "prefer-node-protocol".to_owned(),
                message: format!("Prefer `node:{source}` over `{source}`"),
                span: source_span,
                severity: Severity::Warning,
                help: Some(format!("Add `node:` prefix to `{source}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Replace `{source}` with `node:{source}`"),
                    edits: vec![Edit {
                        span: inner_span,
                        replacement: format!("node:{source}"),
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
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferNodeProtocol)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_bare_fs() {
        let diags = lint("import fs from 'fs';");
        assert_eq!(diags.len(), 1, "bare fs import should be flagged");
    }

    #[test]
    fn test_flags_bare_path() {
        let diags = lint("import path from 'path';");
        assert_eq!(diags.len(), 1, "bare path import should be flagged");
    }

    #[test]
    fn test_allows_node_protocol() {
        let diags = lint("import fs from 'node:fs';");
        assert!(diags.is_empty(), "node: protocol should not be flagged");
    }

    #[test]
    fn test_allows_non_builtin() {
        let diags = lint("import express from 'express';");
        assert!(diags.is_empty(), "non-builtin should not be flagged");
    }

    #[test]
    fn test_flags_subpath() {
        let diags = lint("import { promises } from 'fs/promises';");
        assert_eq!(diags.len(), 1, "subpath without node: should be flagged");
    }
}
