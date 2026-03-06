//! Rule: `typescript/no-empty-interface`
//!
//! Disallow empty interfaces. An empty `interface` with no members and no
//! `extends` clause is equivalent to `{}` (the empty object type) and is
//! almost always a mistake or a leftover from refactoring.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `interface` declarations with no members and no `extends` clause.
#[derive(Debug)]
pub struct NoEmptyInterface;

impl NativeRule for NoEmptyInterface {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-empty-interface".to_owned(),
            description: "Disallow empty interfaces".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSInterfaceDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSInterfaceDeclaration(decl) = kind else {
            return;
        };

        // Interfaces that extend another type are intentional even if empty
        // (e.g. branding patterns, module augmentation).
        if !decl.extends.is_empty() {
            return;
        }

        if decl.body.body.is_empty() {
            // Convert `interface Foo {}` to `type Foo = {}`
            let name = &decl.id.name;
            let replacement = format!("type {name} = {{}}");

            ctx.report(Diagnostic {
                rule_name: "typescript/no-empty-interface".to_owned(),
                message:
                    "Empty interface is equivalent to `{}` — consider removing it or adding members"
                        .to_owned(),
                span: Span::new(decl.span.start, decl.span.end),
                severity: Severity::Warning,
                help: Some("Convert to a type alias".to_owned()),
                fix: Some(Fix {
                    message: format!("Convert to `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(decl.span.start, decl.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyInterface)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_interface() {
        let diags = lint("interface Foo {}");
        assert_eq!(diags.len(), 1, "empty interface should be flagged");
    }

    #[test]
    fn test_allows_interface_with_members() {
        let diags = lint("interface Foo { x: number }");
        assert!(
            diags.is_empty(),
            "interface with members should not be flagged"
        );
    }

    #[test]
    fn test_allows_empty_interface_with_extends() {
        let diags = lint("interface Foo extends Bar {}");
        assert!(
            diags.is_empty(),
            "empty interface with extends should not be flagged"
        );
    }
}
