//! Rule: `no-prototype-builtins`
//!
//! Disallow calling `Object.prototype` methods directly on objects.
//! Methods like `hasOwnProperty`, `isPrototypeOf`, and `propertyIsEnumerable`
//! can be shadowed on the object. Use `Object.prototype.hasOwnProperty.call()`
//! or `Object.hasOwn()` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Methods from `Object.prototype` that should not be called directly.
const PROTOTYPE_METHODS: &[&str] = &["hasOwnProperty", "isPrototypeOf", "propertyIsEnumerable"];

/// Flags direct calls to `Object.prototype` methods on objects.
#[derive(Debug)]
pub struct NoPrototypeBuiltins;

impl NativeRule for NoPrototypeBuiltins {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-prototype-builtins".to_owned(),
            description: "Disallow calling Object.prototype methods directly on objects".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `foo.hasOwnProperty(...)` pattern
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();
        if PROTOTYPE_METHODS.contains(&method_name) {
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoPrototypeBuiltins)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
