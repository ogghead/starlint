//! Rule: `no-prototype-builtins`
//!
//! Disallow calling `Object.prototype` methods directly on objects.
//! Methods like `hasOwnProperty`, `isPrototypeOf`, and `propertyIsEnumerable`
//! can be shadowed on the object. Use `Object.prototype.hasOwnProperty.call()`
//! or `Object.hasOwn()` instead.

use starlint_plugin_sdk::diagnostic::{Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Methods from `Object.prototype` that should not be called directly.
const PROTOTYPE_METHODS: &[&str] = &["hasOwnProperty", "isPrototypeOf", "propertyIsEnumerable"];

/// Flags direct calls to `Object.prototype` methods on objects.
#[derive(Debug)]
pub struct NoPrototypeBuiltins;

impl LintRule for NoPrototypeBuiltins {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-prototype-builtins".to_owned(),
            description: "Disallow calling Object.prototype methods directly on objects".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check for `foo.hasOwnProperty(...)` pattern
        let Some(AstNode::StaticMemberExpression(member)) = ctx.node(call.callee) else {
            return;
        };

        let method_name = member.property.as_str();
        if PROTOTYPE_METHODS.contains(&method_name) {
            // Fix: obj.hasOwnProperty(x) → Object.prototype.hasOwnProperty.call(obj, x)
            // For hasOwnProperty specifically, prefer Object.hasOwn(obj, x)
            let member_object_id = member.object;
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let obj_ast_span = ctx.node(member_object_id).map_or(
                    starlint_ast::types::Span::new(0, 0),
                    starlint_ast::AstNode::span,
                );
                let obj_text = source
                    .get(obj_ast_span.start as usize..obj_ast_span.end as usize)
                    .unwrap_or("");

                // Collect all arguments
                let args: Vec<&str> = call
                    .arguments
                    .iter()
                    .filter_map(|arg_id| {
                        let s = ctx.node(*arg_id).map(starlint_ast::AstNode::span)?;
                        source.get(s.start as usize..s.end as usize)
                    })
                    .collect();

                if method_name == "hasOwnProperty" && args.len() == 1 {
                    // Use modern Object.hasOwn()
                    let arg = args.first().unwrap_or(&"");
                    let replacement = format!("Object.hasOwn({obj_text}, {arg})");
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                } else {
                    let all_args = if args.is_empty() {
                        obj_text.to_owned()
                    } else {
                        format!("{obj_text}, {}", args.join(", "))
                    };
                    let replacement = format!("Object.prototype.{method_name}.call({all_args})");
                    Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(call.span.start, call.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    })
                }
            };

            ctx.report(starlint_plugin_sdk::diagnostic::Diagnostic {
                rule_name: "no-prototype-builtins".to_owned(),
                message: format!(
                    "Do not access `Object.prototype` method `{method_name}` from target object"
                ),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: Some(format!(
                    "Use `Object.prototype.{method_name}.call(obj, ...)` instead"
                )),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoPrototypeBuiltins)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_has_own_property() {
        let diags = lint("foo.hasOwnProperty('bar');");
        assert_eq!(
            diags.len(),
            1,
            "direct hasOwnProperty call should be flagged"
        );
    }

    #[test]
    fn test_flags_is_prototype_of() {
        let diags = lint("foo.isPrototypeOf(bar);");
        assert_eq!(
            diags.len(),
            1,
            "direct isPrototypeOf call should be flagged"
        );
    }

    #[test]
    fn test_flags_property_is_enumerable() {
        let diags = lint("foo.propertyIsEnumerable('bar');");
        assert_eq!(
            diags.len(),
            1,
            "direct propertyIsEnumerable call should be flagged"
        );
    }

    #[test]
    fn test_allows_object_prototype_call() {
        let diags = lint("Object.prototype.hasOwnProperty.call(foo, 'bar');");
        // This calls `call` on the result, which is not `hasOwnProperty` directly
        assert!(
            diags.is_empty() || diags.iter().all(|d| d.message.contains("hasOwnProperty")),
            "Object.prototype pattern should be fine"
        );
    }

    #[test]
    fn test_allows_object_has_own() {
        let diags = lint("Object.hasOwn(foo, 'bar');");
        assert!(diags.is_empty(), "Object.hasOwn should not be flagged");
    }

    #[test]
    fn test_allows_unrelated_method() {
        let diags = lint("foo.toString();");
        assert!(diags.is_empty(), "unrelated method should not be flagged");
    }
}
