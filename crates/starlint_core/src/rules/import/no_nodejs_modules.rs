//! Rule: `import/no-nodejs-modules`
//!
//! Forbid Node.js built-in modules. Useful for browser-only or Deno
//! projects where Node.js builtins are not available.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Known Node.js built-in module names.
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

/// Flags imports from Node.js built-in modules.
#[derive(Debug)]
pub struct NoNodejsModules;

impl LintRule for NoNodejsModules {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-nodejs-modules".to_owned(),
            description: "Forbid Node.js built-in modules".to_owned(),
            category: Category::Correctness,
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

        let source_value = import.source.as_str();

        // Check both bare name (`fs`) and node: protocol (`node:fs`)
        let module_name = source_value.strip_prefix("node:").unwrap_or(source_value);

        // Also strip subpath (e.g. `fs/promises` -> `fs`)
        let base_module = module_name.find('/').map_or(module_name, |pos| {
            module_name.get(..pos).unwrap_or(module_name)
        });

        if NODE_BUILTINS.contains(&base_module) {
            let import_span = Span::new(import.span.start, import.span.end);
            let fix = FixBuilder::new("Remove Node.js module import", FixKind::SuggestionFix)
                .edit(fix_utils::delete_statement(ctx.source_text(), import_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "import/no-nodejs-modules".to_owned(),
                message: format!("Do not import Node.js built-in module `{source_value}`"),
                span: import_span,
                severity: Severity::Warning,
                help: None,
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoNodejsModules)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_bare_node_module() {
        let diags = lint(r#"import fs from "fs";"#);
        assert_eq!(diags.len(), 1, "bare Node.js module should be flagged");
    }

    #[test]
    fn test_flags_node_protocol() {
        let diags = lint(r#"import path from "node:path";"#);
        assert_eq!(diags.len(), 1, "node: protocol import should be flagged");
    }

    #[test]
    fn test_flags_subpath_import() {
        let diags = lint(r#"import { readFile } from "fs/promises";"#);
        assert_eq!(diags.len(), 1, "subpath Node.js import should be flagged");
    }

    #[test]
    fn test_allows_non_node_module() {
        let diags = lint(r#"import lodash from "lodash";"#);
        assert!(diags.is_empty(), "non-Node.js module should not be flagged");
    }
}
