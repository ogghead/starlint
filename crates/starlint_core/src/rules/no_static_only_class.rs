//! Rule: `no-static-only-class`
//!
//! Disallow classes that contain only static members. A class with exclusively
//! static methods and properties is better expressed as a plain object or
//! module-level exports — the `class` keyword adds no value when there is no
//! instantiation or inheritance.

use oxc_ast::AstKind;
use oxc_ast::ast::ClassElement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags classes whose body consists entirely of static members.
#[derive(Debug)]
pub struct NoStaticOnlyClass;

impl NativeRule for NoStaticOnlyClass {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-static-only-class".to_owned(),
            description: "Disallow classes with only static members".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Class])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Skip classes with a superclass — they may rely on inheritance.
        if class.super_class.is_some() {
            return;
        }

        let elements = &class.body.body;

        // Empty classes are not flagged.
        if elements.is_empty() {
            return;
        }

        let all_static = elements.iter().all(|element| match element {
            ClassElement::MethodDefinition(method) => method.r#static,
            ClassElement::PropertyDefinition(prop) => prop.r#static,
            // Static blocks and accessor properties are inherently static.
            ClassElement::StaticBlock(_) | ClassElement::AccessorProperty(_) => true,
            // TSIndexSignature is a TypeScript-only construct; treat as non-static
            // to avoid false positives.
            ClassElement::TSIndexSignature(_) => false,
        });

        if all_static {
            ctx.report(Diagnostic {
                rule_name: "no-static-only-class".to_owned(),
                message: "Class contains only static members — use a plain object or module exports instead".to_owned(),
                span: Span::new(class.span.start, class.span.end),
                severity: Severity::Warning,
                help: None,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoStaticOnlyClass)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_all_static_members() {
        let diags = lint("class Foo { static bar() {} static baz = 1; }");
        assert_eq!(
            diags.len(),
            1,
            "class with only static members should be flagged"
        );
    }

    #[test]
    fn test_flags_single_static_method() {
        let diags = lint("class Foo { static bar() {} }");
        assert_eq!(
            diags.len(),
            1,
            "class with a single static method should be flagged"
        );
    }

    #[test]
    fn test_allows_instance_method() {
        let diags = lint("class Foo { bar() {} }");
        assert!(
            diags.is_empty(),
            "class with instance method should not be flagged"
        );
    }

    #[test]
    fn test_allows_mixed_static_and_instance() {
        let diags = lint("class Foo { static bar() {} baz() {} }");
        assert!(
            diags.is_empty(),
            "class with mixed members should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_class() {
        let diags = lint("class Foo {}");
        assert!(diags.is_empty(), "empty class should not be flagged");
    }

    #[test]
    fn test_allows_class_with_superclass() {
        let diags = lint("class Foo extends Base { static bar() {} }");
        assert!(
            diags.is_empty(),
            "class extending a superclass should not be flagged"
        );
    }

    #[test]
    fn test_allows_static_and_instance_property() {
        let diags = lint("class Foo { static x = 1; y = 2; }");
        assert!(
            diags.is_empty(),
            "class with static and instance properties should not be flagged"
        );
    }
}
