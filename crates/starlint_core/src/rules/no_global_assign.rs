//! Rule: `no-global-assign` (eslint)
//!
//! Disallow assignment to native/global objects. Assigning to built-in
//! globals like `Object`, `Array`, `undefined`, etc. can cause unexpected
//! behavior throughout the application.

use oxc_ast::AstKind;
use oxc_ast::ast::AssignmentTarget;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for NoGlobalAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-global-assign".to_owned(),
            description: "Disallow assignment to native/global objects".to_owned(),
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

        let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left else {
            return;
        };

        let name = id.name.as_str();
        if !READ_ONLY_GLOBALS.contains(&name) {
            return;
        }

        // Only flag if the name is not locally declared (unresolved reference)
        if let Some(semantic) = ctx.semantic() {
            if id
                .reference_id
                .get()
                .and_then(|ref_id| semantic.scoping().get_reference(ref_id).symbol_id())
                .is_some()
            {
                return;
            }
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::{build_semantic, parse_file};
    use crate::traversal::traverse_and_lint_with_semantic;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoGlobalAssign)];
            traverse_and_lint_with_semantic(
                program,
                &rules,
                source,
                Path::new("test.js"),
                Some(&semantic),
            )
        } else {
            vec![]
        }
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
