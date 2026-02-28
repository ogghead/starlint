//! Rule: `import/no-named-as-default-member`
//!
//! Forbid use of an exported name as a property of the default export.
//! For example, if a module has `export const foo = 1; export default bar`,
//! then `import bar from './mod'; bar.foo` should use `import { foo }` instead.
//!
//! Without full module resolution, this rule is a stub that documents intent.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Stub: would flag property access on default imports that matches a named export.
#[derive(Debug)]
pub struct NoNamedAsDefaultMember;

impl NativeRule for NoNamedAsDefaultMember {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-named-as-default-member".to_owned(),
            description: "Forbid use of an exported name as a property of the default export"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, _kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        // Requires cross-module resolution to determine whether property
        // access on a default import matches a named export of the source module.
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamedAsDefaultMember)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_stub_does_not_flag_default_import() {
        let diags = lint(r#"import foo from "./module";"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }

    #[test]
    fn test_stub_does_not_flag_member_access() {
        let diags = lint(r#"import foo from "./module"; console.log(foo.bar);"#);
        assert!(diags.is_empty(), "stub rule should not produce diagnostics");
    }
}
