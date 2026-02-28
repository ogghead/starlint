//! Rule: `typescript/no-misused-new`
//!
//! Disallow `new` in an interface and `constructor` in a type alias.
//! Interfaces should not declare construct signatures (`new(): T`) because
//! they cannot be instantiated with `new` — use a class instead. Similarly,
//! type aliases wrapping object literal types should not contain construct
//! signatures, as this is almost always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::{TSSignature, TSType};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-misused-new";

/// Flags `new()` construct signatures in interfaces and type aliases.
#[derive(Debug)]
pub struct NoMisusedNew;

impl NativeRule for NoMisusedNew {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `new` in an interface and `constructor` in a type alias"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::TSInterfaceDeclaration(decl) => {
                check_interface_body(&decl.body.body, ctx);
            }
            AstKind::TSTypeAliasDeclaration(decl) => {
                if let TSType::TSTypeLiteral(lit) = &decl.type_annotation {
                    check_type_literal_members(&lit.members, ctx);
                }
            }
            _ => {}
        }
    }
}

/// Check interface body members for construct signatures.
fn check_interface_body(members: &[TSSignature<'_>], ctx: &mut NativeLintContext<'_>) {
    for member in members {
        if let TSSignature::TSConstructSignatureDeclaration(sig) = member {
            ctx.report_error(
                RULE_NAME,
                "Interfaces cannot be constructed — use a class instead of `new()` in an interface",
                Span::new(sig.span.start, sig.span.end),
            );
        }
    }
}

/// Check type literal members for construct signatures.
fn check_type_literal_members(members: &[TSSignature<'_>], ctx: &mut NativeLintContext<'_>) {
    for member in members {
        if let TSSignature::TSConstructSignatureDeclaration(sig) = member {
            ctx.report_error(
                RULE_NAME,
                "Type aliases should not contain construct signatures — use a class instead",
                Span::new(sig.span.start, sig.span.end),
            );
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMisusedNew)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_new_in_interface() {
        let diags = lint("interface I { new(): I }");
        assert_eq!(
            diags.len(),
            1,
            "construct signature in interface should be flagged"
        );
    }

    #[test]
    fn test_flags_new_in_type_alias() {
        let diags = lint("type T = { new(): T }");
        assert_eq!(
            diags.len(),
            1,
            "construct signature in type alias should be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_methods() {
        let diags = lint("interface I { foo(): void }");
        assert!(
            diags.is_empty(),
            "interface with regular methods should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_with_constructor() {
        let diags = lint("class C { constructor() {} }");
        assert!(
            diags.is_empty(),
            "class with constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_properties() {
        let diags = lint("interface I { x: number; y: string }");
        assert!(
            diags.is_empty(),
            "interface with properties should not be flagged"
        );
    }
}
