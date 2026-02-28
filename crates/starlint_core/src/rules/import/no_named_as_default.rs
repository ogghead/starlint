//! Rule: `import/no-named-as-default`
//!
//! Forbid use of an exported name as the default import identifier.
//! This is a common mistake when a module exports both a default and named
//! exports, and the developer accidentally uses a named export's name as
//! the default import.
//!
//! Without full module resolution, this rule is a stub that documents intent.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Stub: would flag default imports that share a name with a named export.
#[derive(Debug)]
pub struct NoNamedAsDefault;

impl NativeRule for NoNamedAsDefault {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-as-default".to_owned(),
            description: "Forbid use of an exported name as the default import".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, _kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        // Requires cross-module resolution to determine whether the default
        // import identifier matches a named export of the source module.
        // This is a placeholder for future implementation.
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamedAsDefault)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_stub_does_not_flag_default_import() {
        let diags = lint(r#"import Foo from "./module";"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }

    #[test]
    fn test_stub_does_not_flag_named_import() {
        let diags = lint(r#"import { Foo } from "./module";"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }
}
