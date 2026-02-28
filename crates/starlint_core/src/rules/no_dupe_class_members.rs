//! Rule: `no-dupe-class-members`
//!
//! Disallow duplicate names in class members. Having two methods or properties
//! with the same name in a class body means the second one silently overwrites
//! the first, which is almost always a mistake.

use std::collections::HashSet;

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, MethodDefinitionKind, PropertyKey};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags duplicate method/property names in class bodies.
#[derive(Debug)]
pub struct NoDupeClassMembers;

impl NativeRule for NoDupeClassMembers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-dupe-class-members".to_owned(),
            description: "Disallow duplicate class members".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Track seen member keys as (is_static, name) pairs
        let mut seen: HashSet<(bool, String)> = HashSet::new();

        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };

            // Skip getters and setters — a getter/setter pair with the
            // same name is valid.
            if method.kind == MethodDefinitionKind::Get || method.kind == MethodDefinitionKind::Set
            {
                continue;
            }

            // Skip computed properties — we can't statically determine duplicates.
            if method.computed {
                continue;
            }

            let Some(name) = static_key_name(&method.key) else {
                continue;
            };

            let key = (method.r#static, name);
            if !seen.insert(key) {
                ctx.report_error(
                    "no-dupe-class-members",
                    &format!(
                        "Duplicate class member `{}`",
                        static_key_name(&method.key).unwrap_or_default()
                    ),
                    Span::new(method.span.start, method.span.end),
                );
            }
        }
    }
}

/// Extract the name of a non-computed property key.
fn static_key_name(key: &PropertyKey<'_>) -> Option<String> {
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDupeClassMembers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_methods() {
        let diags = lint("class Foo { bar() {} bar() {} }");
        assert_eq!(diags.len(), 1, "duplicate method should be flagged");
    }

    #[test]
    fn test_allows_different_methods() {
        let diags = lint("class Foo { bar() {} baz() {} }");
        assert!(diags.is_empty(), "different methods should not be flagged");
    }

    #[test]
    fn test_allows_getter_setter_pair() {
        let diags = lint("class Foo { get bar() {} set bar(v) {} }");
        assert!(diags.is_empty(), "getter/setter pair should not be flagged");
    }

    #[test]
    fn test_flags_duplicate_static_methods() {
        let diags = lint("class Foo { static bar() {} static bar() {} }");
        assert_eq!(diags.len(), 1, "duplicate static methods should be flagged");
    }

    #[test]
    fn test_allows_static_and_instance_same_name() {
        let diags = lint("class Foo { bar() {} static bar() {} }");
        assert!(
            diags.is_empty(),
            "static and instance with same name should not be flagged"
        );
    }

    #[test]
    fn test_allows_constructor() {
        let diags = lint("class Foo { constructor() {} bar() {} }");
        assert!(
            diags.is_empty(),
            "constructor with other methods should not be flagged"
        );
    }
}
