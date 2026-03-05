//! Rule: `no-extend-native`
//!
//! Disallow extending native types via prototype. Modifying
//! `Object.prototype`, `Array.prototype`, etc. is dangerous as it
//! can break third-party code and create unexpected behavior.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags assignments to native type prototypes.
#[derive(Debug)]
pub struct NoExtendNative;

/// Built-in JS constructor names whose prototypes should not be extended.
const NATIVE_TYPES: &[&str] = &[
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Date",
    "RegExp",
    "Error",
    "Function",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "Promise",
    "Symbol",
    "ArrayBuffer",
    "DataView",
    "Float32Array",
    "Float64Array",
    "Int8Array",
    "Int16Array",
    "Int32Array",
    "Uint8Array",
    "Uint16Array",
    "Uint32Array",
    "BigInt",
    "BigInt64Array",
    "BigUint64Array",
];

impl NativeRule for NoExtendNative {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extend-native".to_owned(),
            description: "Disallow extending native types".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Match: NativeType.prototype.foo = ... or NativeType.prototype = ...
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Check the source text of the left side for "NativeType.prototype"
        let source = ctx.source_text();
        let target_span = assign.left.span();
        let start = usize::try_from(target_span.start).unwrap_or(0);
        let end = usize::try_from(target_span.end).unwrap_or(0);
        let target_text = source.get(start..end).unwrap_or("");
        let span_start = assign.span.start;
        let span_end = assign.span.end;

        for native in NATIVE_TYPES {
            let prefix = format!("{native}.prototype");
            if target_text.starts_with(&prefix) {
                ctx.report(Diagnostic {
                    rule_name: "no-extend-native".to_owned(),
                    message: format!(
                        "{native} prototype is read only, properties should not be added"
                    ),
                    span: Span::new(span_start, span_end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
                return;
            }
        }
    }
}

use oxc_span::GetSpan;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtendNative)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_prototype_extension() {
        let diags = lint("Object.prototype.foo = function() {};");
        assert_eq!(
            diags.len(),
            1,
            "Object.prototype extension should be flagged"
        );
    }

    #[test]
    fn test_flags_array_prototype_extension() {
        let diags = lint("Array.prototype.flat2 = function() {};");
        assert_eq!(
            diags.len(),
            1,
            "Array.prototype extension should be flagged"
        );
    }

    #[test]
    fn test_allows_custom_prototype() {
        let diags = lint("MyClass.prototype.foo = function() {};");
        assert!(diags.is_empty(), "custom prototype should not be flagged");
    }

    #[test]
    fn test_allows_normal_assignment() {
        let diags = lint("var x = 5;");
        assert!(diags.is_empty(), "normal assignment should not be flagged");
    }
}
