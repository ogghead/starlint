//! Rule: `node/no-exports-assign`
//!
//! Disallow direct assignment to `exports`. In `CommonJS`, the `exports`
//! variable is a reference to `module.exports`. Reassigning `exports`
//! directly (e.g. `exports = {}`) breaks that reference and does not
//! change what the module actually exports.

use oxc_ast::AstKind;
use oxc_ast::ast::AssignmentTarget;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags direct assignment to the `exports` identifier.
///
/// `exports = value` breaks the module reference. Use
/// `module.exports = value` or `exports.prop = value` instead.
#[derive(Debug)]
pub struct NoExportsAssign;

impl NativeRule for NoExportsAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "node/no-exports-assign".to_owned(),
            description: "Disallow direct assignment to `exports`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Only flag bare `exports = ...` (identifier target).
        // `exports.foo = bar` is fine (extending, not reassigning).
        let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left else {
            return;
        };

        if id.name.as_str() != "exports" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "node/no-exports-assign".to_owned(),
            message: "Direct assignment to `exports` breaks the module reference \u{2014} use `module.exports` or `exports.prop` instead".to_owned(),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Error,
            help: Some("Replace `exports` with `module.exports`".to_owned()),
            fix: Some(Fix {
                message: "Replace `exports` with `module.exports`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(id.span.start, id.span.end),
                    replacement: "module.exports".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExportsAssign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_exports_reassignment() {
        let diags = lint("exports = {};");
        assert_eq!(
            diags.len(),
            1,
            "direct assignment to exports should be flagged"
        );
    }

    #[test]
    fn test_flags_exports_assign_variable() {
        let diags = lint("exports = something;");
        assert_eq!(
            diags.len(),
            1,
            "assigning variable to exports should be flagged"
        );
    }

    #[test]
    fn test_allows_exports_property_assignment() {
        let diags = lint("exports.foo = bar;");
        assert!(diags.is_empty(), "exports.foo = bar should not be flagged");
    }

    #[test]
    fn test_allows_module_exports_assignment() {
        let diags = lint("module.exports = {};");
        assert!(
            diags.is_empty(),
            "module.exports assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_assignment() {
        let diags = lint("x = 1;");
        assert!(diags.is_empty(), "normal assignment should not be flagged");
    }
}
