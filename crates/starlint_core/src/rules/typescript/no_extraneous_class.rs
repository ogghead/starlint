//! Rule: `typescript/no-extraneous-class`
//!
//! Disallow classes that contain only static members or are empty. Such classes
//! add no value over plain objects or module-level functions and exports. If a
//! class has a constructor, extends another class, or contains any instance
//! members, it is considered valid.

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, MethodDefinitionKind};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags classes that are empty or contain only static members.
#[derive(Debug)]
pub struct NoExtraneousClass;

impl NativeRule for NoExtraneousClass {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-extraneous-class".to_owned(),
            description: "Disallow classes with only static members or empty bodies".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Class])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Classes that extend another class may rely on inheritance behavior.
        if class.super_class.is_some() {
            return;
        }

        let elements = &class.body.body;

        // Check if the class has a constructor or any instance member.
        if has_constructor_or_instance_member(elements) {
            return;
        }

        // At this point the class is either empty or all-static — flag it.
        let message = if elements.is_empty() {
            "Empty class is unnecessary — use an object literal or remove it"
        } else {
            "Class contains only static members — use a plain object or module-level functions instead"
        };

        ctx.report_warning(
            "typescript/no-extraneous-class",
            message,
            Span::new(class.span.start, class.span.end),
        );
    }
}

/// Check whether a class body contains a constructor or any instance (non-static) member.
fn has_constructor_or_instance_member(elements: &[ClassElement<'_>]) -> bool {
    for element in elements {
        match element {
            ClassElement::MethodDefinition(method) => {
                if method.kind == MethodDefinitionKind::Constructor {
                    return true;
                }
                if !method.r#static {
                    return true;
                }
            }
            ClassElement::PropertyDefinition(prop) => {
                if !prop.r#static {
                    return true;
                }
            }
            ClassElement::AccessorProperty(acc) => {
                if !acc.r#static {
                    return true;
                }
            }
            // Static blocks are inherently static; TSIndexSignature is a TS
            // construct that we treat as instance-like to avoid false positives.
            ClassElement::StaticBlock(_) => {}
            ClassElement::TSIndexSignature(_) => {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraneousClass)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_static_only_class() {
        let diags = lint("class C { static foo() {} }");
        assert_eq!(
            diags.len(),
            1,
            "class with only static members should be flagged"
        );
    }

    #[test]
    fn test_flags_empty_class() {
        let diags = lint("class C {}");
        assert_eq!(diags.len(), 1, "empty class should be flagged");
    }

    #[test]
    fn test_allows_class_with_instance_method() {
        let diags = lint("class C { foo() {} }");
        assert!(
            diags.is_empty(),
            "class with instance method should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_with_extends() {
        let diags = lint("class C extends Base {}");
        assert!(
            diags.is_empty(),
            "class extending a base class should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_with_constructor() {
        let diags = lint("class C { constructor() {} }");
        assert!(
            diags.is_empty(),
            "class with a constructor should not be flagged"
        );
    }
}
