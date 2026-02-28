//! Rule: `typescript/prefer-function-type`
//!
//! Prefer function type syntax (`() => void`) over an interface or object type
//! with a single call signature. When an interface has exactly one member and
//! that member is a call signature, the interface can be replaced with a simpler
//! function type alias, improving readability.

use oxc_ast::AstKind;
use oxc_ast::ast::TSSignature;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-function-type";

/// Flags interfaces whose body contains exactly one member that is a call
/// signature declaration — these can be expressed more concisely as function
/// type aliases.
#[derive(Debug)]
pub struct PreferFunctionType;

impl NativeRule for PreferFunctionType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer function type syntax over interface with a single call signature"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSInterfaceDeclaration(decl) = kind else {
            return;
        };

        // Interfaces that extend other types are intentional even if they have
        // a single call signature (they add the callable behavior to the base).
        if !decl.extends.is_empty() {
            return;
        }

        let members = &decl.body.body;

        // Only flag when there is exactly one member
        if members.len() != 1 {
            return;
        }

        // Check if the single member is a call signature
        let Some(member) = members.first() else {
            return;
        };

        if matches!(member, TSSignature::TSCallSignatureDeclaration(_)) {
            let name = decl.id.name.as_str();
            ctx.report_warning(
                RULE_NAME,
                &format!(
                    "Interface `{name}` has only a call signature — use a function type instead (e.g. `type {name} = (...) => ...`)"
                ),
                Span::new(decl.span.start, decl.span.end),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferFunctionType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_interface_with_single_call_signature() {
        let diags = lint("interface Foo { (): void }");
        assert_eq!(
            diags.len(),
            1,
            "interface with only a call signature should be flagged"
        );
    }

    #[test]
    fn test_flags_interface_with_parameterized_call_signature() {
        let diags = lint("interface Callback { (x: number, y: string): boolean }");
        assert_eq!(
            diags.len(),
            1,
            "interface with a parameterized call signature should be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_multiple_members() {
        let diags = lint("interface Foo { (): void; bar: string }");
        assert!(
            diags.is_empty(),
            "interface with call signature and other members should not be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_extends() {
        let diags = lint("interface Foo extends Bar { (): void }");
        assert!(
            diags.is_empty(),
            "interface extending another type should not be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_properties_only() {
        let diags = lint("interface Foo { x: number; y: string }");
        assert!(
            diags.is_empty(),
            "interface with only properties should not be flagged"
        );
    }
}
