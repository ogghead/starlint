//! Rule: `no-class-assign` (eslint)
//!
//! Disallow reassignment of class declarations. Reassigning a class
//! name is almost always a mistake.

use oxc_ast::AstKind;
use oxc_semantic::SymbolFlags;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags reassignment of class declarations.
#[derive(Debug)]
pub struct NoClassAssign;

impl NativeRule for NoClassAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-class-assign".to_owned(),
            description: "Disallow reassignment of class declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Only check class declarations (not expressions)
        if !class.is_declaration() {
            return;
        }

        let Some(id) = &class.id else {
            return;
        };

        let Some(symbol_id) = id.symbol_id.get() else {
            return;
        };

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        // Check that this symbol has the Class flag
        let flags = scoping.symbol_flags(symbol_id);
        if !flags.contains(SymbolFlags::Class) {
            return;
        }

        // Check if any reference to this symbol is a write
        let has_write = scoping
            .get_resolved_references(symbol_id)
            .any(oxc_semantic::Reference::is_write);

        if has_write {
            ctx.report_error(
                "no-class-assign",
                &format!(
                    "'{}' is a class declaration and should not be reassigned",
                    id.name
                ),
                Span::new(id.span.start, id.span.end),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::{build_semantic, parse_file};
    use crate::traversal::traverse_and_lint_with_semantic;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let program = allocator.alloc(parsed.program);
            let semantic = build_semantic(program);
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoClassAssign)];
            traverse_and_lint_with_semantic(
                program,
                &rules,
                source,
                Path::new("test.js"),
                Some(&semantic),
            )
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_class_reassignment() {
        let diags = lint("class Foo {} Foo = bar;");
        assert_eq!(
            diags.len(),
            1,
            "reassigning class declaration should be flagged"
        );
    }

    #[test]
    fn test_allows_class_instantiation() {
        let diags = lint("class Foo {} new Foo();");
        assert!(
            diags.is_empty(),
            "instantiating class should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_expression_reassignment() {
        let diags = lint("var Foo = class {}; Foo = bar;");
        assert!(
            diags.is_empty(),
            "reassigning class expression should not be flagged by this rule"
        );
    }

    #[test]
    fn test_allows_different_name() {
        let diags = lint("class Foo {} bar = baz;");
        assert!(
            diags.is_empty(),
            "assigning different name should not be flagged"
        );
    }
}
