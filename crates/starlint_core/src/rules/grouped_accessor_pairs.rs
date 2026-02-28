//! Rule: `grouped-accessor-pairs`
//!
//! Require getter and setter pairs for the same property to be adjacent in
//! object literals and class bodies. Separating a getter and setter with
//! unrelated properties makes the code harder to read and maintain.

use std::collections::HashMap;

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, MethodDefinitionKind, PropertyKey, PropertyKind};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags getter/setter pairs that are not adjacent.
#[derive(Debug)]
pub struct GroupedAccessorPairs;

impl NativeRule for GroupedAccessorPairs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "grouped-accessor-pairs".to_owned(),
            description: "Require grouped getter/setter pairs in objects and classes".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
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

/// Tracks the position and span of an accessor (getter or setter).
struct AccessorInfo {
    /// Position of the getter in the property list.
    getter_pos: Option<usize>,
    /// Position of the setter in the property list.
    setter_pos: Option<usize>,
    /// Span of the second accessor found (used for the diagnostic location).
    second_span: Option<Span>,
}

/// Check getter/setter adjacency in an object expression.
fn check_object_expression(
    obj: &oxc_ast::ast::ObjectExpression<'_>,
    ctx: &mut NativeLintContext<'_>,
) {
    let mut accessors: HashMap<String, AccessorInfo> = HashMap::new();

    for (i, property) in obj.properties.iter().enumerate() {
        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) = property else {
            continue;
        };

        // Only interested in getters and setters
        if prop.kind != PropertyKind::Get && prop.kind != PropertyKind::Set {
            continue;
        }

        // Skip computed properties — can't determine name statically
        if prop.computed {
            continue;
        }

        let Some(name) = static_property_key_name(&prop.key) else {
            continue;
        };

        let entry = accessors.entry(name).or_insert_with(|| AccessorInfo {
            getter_pos: None,
            setter_pos: None,
            second_span: None,
        });

        if prop.kind == PropertyKind::Get {
            entry.getter_pos = Some(i);
        } else {
            entry.setter_pos = Some(i);
        }

        // If both are now set, record the span of this (second) accessor
        if entry.getter_pos.is_some() && entry.setter_pos.is_some() && entry.second_span.is_none() {
            entry.second_span = Some(Span::new(prop.span.start, prop.span.end));
        }
    }

    report_non_adjacent(&accessors, ctx);
}

/// Check getter/setter adjacency in a class body.
fn check_class_body(class: &oxc_ast::ast::Class<'_>, ctx: &mut NativeLintContext<'_>) {
    let mut accessors: HashMap<(bool, String), AccessorInfo> = HashMap::new();

    for (i, element) in class.body.body.iter().enumerate() {
        let ClassElement::MethodDefinition(method) = element else {
            continue;
        };

        // Only interested in getters and setters
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
        let entry = accessors.entry(key).or_insert_with(|| AccessorInfo {
            getter_pos: None,
            setter_pos: None,
            second_span: None,
        });

        if method.kind == MethodDefinitionKind::Get {
            entry.getter_pos = Some(i);
        } else {
            entry.setter_pos = Some(i);
        }

        // If both are now set, record the span of this (second) accessor
        if entry.getter_pos.is_some() && entry.setter_pos.is_some() && entry.second_span.is_none() {
            entry.second_span = Some(Span::new(method.span.start, method.span.end));
        }
    }

    report_non_adjacent_keyed(&accessors, ctx);
}

/// Report diagnostics for non-adjacent getter/setter pairs (string keys).
fn report_non_adjacent(accessors: &HashMap<String, AccessorInfo>, ctx: &mut NativeLintContext<'_>) {
    for (name, info) in accessors {
        let (Some(getter_pos), Some(setter_pos)) = (info.getter_pos, info.setter_pos) else {
            continue;
        };

        let diff = getter_pos.abs_diff(setter_pos);
        if diff > 1 {
            if let Some(span) = info.second_span {
                ctx.report_warning(
                    "grouped-accessor-pairs",
                    &format!("Getter and setter for `{name}` should be grouped together"),
                    span,
                );
            }
        }
    }
}

/// Report diagnostics for non-adjacent getter/setter pairs (static+name keys).
fn report_non_adjacent_keyed(
    accessors: &HashMap<(bool, String), AccessorInfo>,
    ctx: &mut NativeLintContext<'_>,
) {
    for ((_, name), info) in accessors {
        let (Some(getter_pos), Some(setter_pos)) = (info.getter_pos, info.setter_pos) else {
            continue;
        };

        let diff = getter_pos.abs_diff(setter_pos);
        if diff > 1 {
            if let Some(span) = info.second_span {
                ctx.report_warning(
                    "grouped-accessor-pairs",
                    &format!("Getter and setter for `{name}` should be grouped together"),
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

    /// Helper to lint source code with the `GroupedAccessorPairs` rule.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GroupedAccessorPairs)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_adjacent_object_accessors() {
        let diags = lint("const obj = { get foo() {}, set foo(v) {} }");
        assert!(
            diags.is_empty(),
            "adjacent getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_non_adjacent_object_accessors() {
        let diags = lint("const obj = { get foo() {}, bar: 1, set foo(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent getter/setter pair should be flagged"
        );
    }

    #[test]
    fn test_allows_adjacent_class_accessors() {
        let diags = lint("class C { get foo() {} set foo(v) {} }");
        assert!(
            diags.is_empty(),
            "adjacent class getter/setter should not be flagged"
        );
    }

    #[test]
    fn test_flags_non_adjacent_class_accessors() {
        let diags = lint("class C { get foo() {} bar() {} set foo(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent class getter/setter should be flagged"
        );
    }

    #[test]
    fn test_allows_only_getter() {
        let diags = lint("const obj = { get foo() {} }");
        assert!(diags.is_empty(), "lone getter should not be flagged");
    }

    #[test]
    fn test_allows_only_setter() {
        let diags = lint("const obj = { set foo(v) {} }");
        assert!(diags.is_empty(), "lone setter should not be flagged");
    }

    #[test]
    fn test_allows_reversed_adjacent() {
        let diags = lint("const obj = { set foo(v) {}, get foo() {} }");
        assert!(
            diags.is_empty(),
            "reversed but adjacent getter/setter should not be flagged"
        );
    }

    #[test]
    fn test_flags_static_non_adjacent() {
        let diags = lint("class C { static get x() {} bar() {} static set x(v) {} }");
        assert_eq!(
            diags.len(),
            1,
            "non-adjacent static getter/setter should be flagged"
        );
    }
}
