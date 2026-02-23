//! Rule: `no-extend-native`
//!
//! Disallow extending native types via prototype. Modifying
//! `Object.prototype`, `Array.prototype`, etc. is dangerous as it
//! can break third-party code and create unexpected behavior.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

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

impl LintRule for NoExtendNative {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extend-native".to_owned(),
            description: "Disallow extending native types".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::AssignmentExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Match: NativeType.prototype.foo = ... or NativeType.prototype = ...
        let AstNode::AssignmentExpression(assign) = node else {
            return;
        };

        // Check the source text of the left side for "NativeType.prototype"
        let source = ctx.source_text();
        let Some(left_node) = ctx.node(assign.left) else {
            return;
        };
        let target_span = left_node.span();
        let start = usize::try_from(target_span.start).unwrap_or(0);
        let end = usize::try_from(target_span.end).unwrap_or(0);
        let target_text = source.get(start..end).unwrap_or("");
        let span_start = assign.span.start;
        let span_end = assign.span.end;

        for native in NATIVE_TYPES {
            if target_text.starts_with(native)
                && target_text
                    .get(native.len()..)
                    .is_some_and(|rest| rest.starts_with(".prototype"))
            {
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

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoExtendNative)];
        lint_source(source, "test.js", &rules)
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
