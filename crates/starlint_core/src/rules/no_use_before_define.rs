//! Rule: `no-use-before-define` (eslint)
//!
//! Disallow the use of variables before they are defined. This helps
//! avoid confusion and ensures code reads top-to-bottom.

use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags references to variables used before their declaration.
#[derive(Debug)]
pub struct NoUseBeforeDefine;

impl NativeRule for NoUseBeforeDefine {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-use-before-define".to_owned(),
            description: "Disallow use of variables before they are defined".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Check let/const declarations — they have a temporal dead zone
        let AstKind::VariableDeclaration(decl) = kind else {
            return;
        };

        // Only let/const have TDZ issues
        if decl.kind == VariableDeclarationKind::Var {
            return;
        }

        let Some(semantic) = ctx.semantic() else {
            return;
        };

        let scoping = semantic.scoping();

        for declarator in &decl.declarations {
            let binding_ids = declarator.id.get_binding_identifiers();

            for binding in &binding_ids {
                let Some(symbol_id) = binding.symbol_id.get() else {
                    continue;
                };

                // Check if any reference to this symbol comes before the declaration
                for reference in scoping.get_resolved_references(symbol_id) {
                    let ref_span = semantic.reference_span(reference);
                    if ref_span.start < binding.span.start {
                        ctx.report_warning(
                            "no-use-before-define",
                            &format!("'{}' is used before it is defined", binding.name),
                            Span::new(ref_span.start, ref_span.end),
                        );
                    }
                }
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUseBeforeDefine)];
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
    fn test_allows_use_after_define() {
        let diags = lint("const x = 1; foo(x);");
        assert!(diags.is_empty(), "use after define should not be flagged");
    }

    #[test]
    fn test_allows_var_hoisting() {
        let diags = lint("foo(x); var x = 1;");
        assert!(diags.is_empty(), "var hoisting should not be flagged");
    }
}
