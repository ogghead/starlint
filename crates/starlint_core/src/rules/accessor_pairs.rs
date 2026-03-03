//! Rule: `accessor-pairs`
//!
//! Require matching getter and setter pairs in object literals and class
//! bodies. A setter without a corresponding getter is usually a mistake,
//! because the value can be set but never retrieved.
//!
//! By default (matching `ESLint`), only setters without getters are flagged.
//! Getters without setters are allowed.

use std::collections::HashMap;

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, MethodDefinitionKind, PropertyKey, PropertyKind};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags setters without a corresponding getter.
#[derive(Debug)]
pub struct AccessorPairs;

impl NativeRule for AccessorPairs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "accessor-pairs".to_owned(),
            description: "Require matching getter/setter pairs".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Class, AstType::ObjectExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::ObjectExpression(obj) => {
                check_object_expression(obj, ctx);
            }
            AstKind::Class(class) => {
                check_class_body(class, ctx);
            }
            _ => {}
        }
    }
}

/// Tracks whether a getter and/or setter exists for a property name.
struct PairInfo {
    /// Whether a getter exists for this property.
    has_getter: bool,
    /// Whether a setter exists for this property.
    has_setter: bool,
    /// Span of the setter (used for the diagnostic location when getter is missing).
    setter_span: Option<Span>,
}

/// Check accessor pairs in an object expression.
fn check_object_expression(
    obj: &oxc_ast::ast::ObjectExpression<'_>,
    ctx: &mut NativeLintContext<'_>,
) {
    let mut pairs: HashMap<String, PairInfo> = HashMap::new();

    for property in &obj.properties {
        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) = property else {
            continue;
        };

        if prop.kind != PropertyKind::Get && prop.kind != PropertyKind::Set {
            continue;
        }

        // Skip computed properties
        if prop.computed {
            continue;
        }

        let Some(name) = static_property_key_name(&prop.key) else {
            continue;
        };

        let entry = pairs.entry(name).or_insert_with(|| PairInfo {
            has_getter: false,
            has_setter: false,
            setter_span: None,
        });

        if prop.kind == PropertyKind::Get {
            entry.has_getter = true;
        } else {
            entry.has_setter = true;
            entry.setter_span = Some(Span::new(prop.span.start, prop.span.end));
        }
    }

    report_missing_getters(&pairs, ctx);
}

/// Check accessor pairs in a class body.
fn check_class_body(class: &oxc_ast::ast::Class<'_>, ctx: &mut NativeLintContext<'_>) {
    // Key: (is_static, name)
    let mut pairs: HashMap<(bool, String), PairInfo> = HashMap::new();

    for element in &class.body.body {
        let ClassElement::MethodDefinition(method) = element else {
            continue;
        };

        if method.kind != MethodDefinitionKind::Get && method.kind != MethodDefinitionKind::Set {
            continue;
        }

        // Skip computed properties
        if method.computed {
            continue;
        }

        let Some(name) = static_property_key_name(&method.key) else {
            continue;
        };

        let key = (method.r#static, name);
        let entry = pairs.entry(key).or_insert_with(|| PairInfo {
            has_getter: false,
            has_setter: false,
            setter_span: None,
        });

        if method.kind == MethodDefinitionKind::Get {
            entry.has_getter = true;
        } else {
            entry.has_setter = true;
            entry.setter_span = Some(Span::new(method.span.start, method.span.end));
        }
    }

    report_missing_getters_keyed(&pairs, ctx);
}

/// Report setters that have no matching getter (string keys).
fn report_missing_getters(pairs: &HashMap<String, PairInfo>, ctx: &mut NativeLintContext<'_>) {
    for (name, info) in pairs {
        if info.has_setter && !info.has_getter {
            if let Some(span) = info.setter_span {
                ctx.report_warning(
                    "accessor-pairs",
                    &format!("Setter for `{name}` has no corresponding getter"),
                    span,
                );
            }
        }
    }
}

/// Report setters that have no matching getter (static+name keys).
fn report_missing_getters_keyed(
    pairs: &HashMap<(bool, String), PairInfo>,
    ctx: &mut NativeLintContext<'_>,
) {
    for ((_, name), info) in pairs {
        if info.has_setter && !info.has_getter {
            if let Some(span) = info.setter_span {
                ctx.report_warning(
                    "accessor-pairs",
                    &format!("Setter for `{name}` has no corresponding getter"),
                    span,
                );
            }
        }
    }
}

/// Extract a static key name from a property key.
fn static_property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
        PropertyKey::NumericLiteral(lit) => Some(lit.raw_str().to_string()),
        PropertyKey::PrivateIdentifier(ident) => Some(ident.name.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code with the `AccessorPairs` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(AccessorPairs)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_complete_pair() {
        let diags = lint("const obj = { get foo() {}, set foo(v) {} }");
        assert!(
            diags.is_empty(),
            "complete getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_setter_without_getter() {
        let diags = lint("const obj = { set foo(v) {} }");
        assert_eq!(diags.len(), 1, "setter without getter should be flagged");
    }

    #[test]
    fn test_allows_getter_without_setter() {
        let diags = lint("const obj = { get foo() {} }");
        assert!(
            diags.is_empty(),
            "getter without setter should not be flagged (ESLint default)"
        );
    }

    #[test]
    fn test_allows_class_complete_pair() {
        let diags = lint("class C { get x() {} set x(v) {} }");
        assert!(
            diags.is_empty(),
            "class with complete getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_class_setter_without_getter() {
        let diags = lint("class C { set x(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "class setter without getter should be flagged"
        );
    }

    #[test]
    fn test_allows_class_getter_without_setter() {
        let diags = lint("class C { get x() {} }");
        assert!(
            diags.is_empty(),
            "class getter without setter should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_incomplete_pairs() {
        let diags = lint("const obj = { set foo(v) {}, set bar(v) {} }");
        assert_eq!(
            diags.len(),
            2,
            "two setters without getters should produce two diagnostics"
        );
    }

    #[test]
    fn test_allows_normal_properties() {
        let diags = lint("const obj = { foo: 1, bar: 2 }");
        assert!(diags.is_empty(), "normal properties should not be flagged");
    }

    #[test]
    fn test_flags_static_setter_without_getter() {
        let diags = lint("class C { static set x(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "static setter without getter should be flagged"
        );
    }
}
