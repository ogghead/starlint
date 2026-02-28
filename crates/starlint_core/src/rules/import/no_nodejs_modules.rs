//! Rule: `import/no-nodejs-modules`
//!
//! Forbid Node.js built-in modules. Useful for browser-only or Deno
//! projects where Node.js builtins are not available.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for NoNodejsModules {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-nodejs-modules".to_owned(),
            description: "Forbid Node.js built-in modules".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source_value = import.source.value.as_str();

        // Check both bare name (`fs`) and node: protocol (`node:fs`)
        let module_name = source_value.strip_prefix("node:").unwrap_or(source_value);

        // Also strip subpath (e.g. `fs/promises` -> `fs`)
        let base_module = module_name.find('/').map_or(module_name, |pos| {
            module_name.get(..pos).unwrap_or(module_name)
        });

        if NODE_BUILTINS.contains(&base_module) {
            ctx.report_warning(
                "import/no-nodejs-modules",
                &format!("Do not import Node.js built-in module `{source_value}`"),
                Span::new(import.span.start, import.span.end),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNodejsModules)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
