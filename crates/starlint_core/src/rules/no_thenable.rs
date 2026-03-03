//! Rule: `no-thenable` (unicorn)
//!
//! Disallow the use of `then` as a property name on objects/classes.
//! Objects with a `then` method are treated as "thenables" by the
//! Promise system, which can cause unexpected behavior.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags objects and classes that define a `then` property or method.
#[derive(Debug)]
pub struct NoThenable;

impl NativeRule for NoThenable {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-thenable".to_owned(),
            description: "Disallow then property".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::MethodDefinition,
            AstType::ObjectExpression,
            AstType::PropertyDefinition,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // Check object properties: { then: ... } or { then() {} }
            AstKind::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) = prop else {
                        continue;
                    };
                    if property_key_is_then(&p.key) {
                        ctx.report_warning(
                            "no-thenable",
                            "Do not add `then` to an object",
                            Span::new(p.span.start, p.span.end),
                        );
                    }
                }
            }
            // Check class methods/properties named `then`
            AstKind::MethodDefinition(method) => {
                if property_key_is_then(&method.key) {
                    ctx.report_warning(
                        "no-thenable",
                        "Do not add `then` to a class",
                        Span::new(method.span.start, method.span.end),
                    );
                }
            }
            AstKind::PropertyDefinition(prop) => {
                if property_key_is_then(&prop.key) {
                    ctx.report_warning(
                        "no-thenable",
                        "Do not add `then` to a class",
                        Span::new(prop.span.start, prop.span.end),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Check if a property key is the identifier or string `"then"`.
fn property_key_is_then(key: &oxc_ast::ast::PropertyKey<'_>) -> bool {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => id.name == "then",
        oxc_ast::ast::PropertyKey::StringLiteral(s) => s.value == "then",
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThenable)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_then_property() {
        let diags = lint("var obj = { then: function() {} };");
        assert_eq!(
            diags.len(),
            1,
            "object with then property should be flagged"
        );
    }

    #[test]
    fn test_flags_object_then_method() {
        let diags = lint("var obj = { then() {} };");
        assert_eq!(diags.len(), 1, "object with then method should be flagged");
    }

    #[test]
    fn test_flags_class_then_method() {
        let diags = lint("class Foo { then() {} }");
        assert_eq!(diags.len(), 1, "class with then method should be flagged");
    }

    #[test]
    fn test_allows_other_property() {
        let diags = lint("var obj = { foo: 1 };");
        assert!(diags.is_empty(), "other properties should not be flagged");
    }

    #[test]
    fn test_allows_other_method() {
        let diags = lint("class Foo { bar() {} }");
        assert!(diags.is_empty(), "other methods should not be flagged");
    }
}
