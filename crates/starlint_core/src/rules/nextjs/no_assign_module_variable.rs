//! Rule: `nextjs/no-assign-module-variable`
//!
//! Forbid assigning to the `module` variable, which interferes with
//! Next.js module handling and hot module replacement.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-assign-module-variable";

/// Flags assignment expressions where the left side is the `module` variable.
#[derive(Debug)]
pub struct NoAssignModuleVariable;

impl NativeRule for NoAssignModuleVariable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid assigning to the `module` variable".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        let is_module_target = match &assign.left {
            oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                ident.name.as_str() == "module"
            }
            _ => false,
        };

        if is_module_target {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not assign to the `module` variable -- it interferes with Next.js module handling".to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAssignModuleVariable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_module_assignment() {
        let diags = lint("module = { exports: {} };");
        assert_eq!(diags.len(), 1, "module assignment should be flagged");
    }

    #[test]
    fn test_allows_module_exports() {
        let diags = lint("module.exports = {};");
        assert!(diags.is_empty(), "module.exports should not be flagged");
    }

    #[test]
    fn test_allows_other_variable() {
        let diags = lint("let x = 1;");
        assert!(
            diags.is_empty(),
            "other variable assignment should not be flagged"
        );
    }
}
