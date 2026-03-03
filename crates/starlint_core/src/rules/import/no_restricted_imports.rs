//! Rule: `import/no-restricted-imports`
//!
//! Forbid specific modules when loaded by import.
//! This is a stub rule — the list of restricted modules would be provided
//! via configuration.

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::NativeRule;

/// Stub: would forbid specific configured modules from being imported.
#[derive(Debug)]
pub struct NoRestrictedImports;

impl NativeRule for NoRestrictedImports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-restricted-imports".to_owned(),
            description: "Forbid specific modules when loaded by import".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestrictedImports)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_stub_does_not_flag_any_import() {
        let diags = lint(r#"import foo from "lodash";"#);
        assert!(
            diags.is_empty(),
            "stub rule should not produce diagnostics without configuration"
        );
    }

    #[test]
    fn test_stub_does_not_flag_named_import() {
        let diags = lint(r#"import { get } from "lodash";"#);
        assert!(
            diags.is_empty(),
            "stub rule should not produce diagnostics without configuration"
        );
    }
}
