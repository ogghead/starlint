//! Rule: `no-var`
//!
//! Disallow `var` declarations. Prefer `let` and `const` which have
//! block-scoped semantics and avoid common hoisting bugs.

use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `var` declarations, suggesting `let` instead.
#[derive(Debug)]
pub struct NoVar;

impl NativeRule for NoVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-var".to_owned(),
            description: "Require `let` or `const` instead of `var`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::VariableDeclaration(decl) = kind {
            if decl.kind == VariableDeclarationKind::Var {
                // The `var` keyword is always the first 3 bytes of the declaration span.
                let var_span = Span::new(decl.span.start, decl.span.start.saturating_add(3));

                ctx.report(Diagnostic {
                    rule_name: "no-var".to_owned(),
                    message: "Unexpected `var`, use `let` or `const` instead".to_owned(),
                    span: Span::new(decl.span.start, decl.span.end),
                    severity: Severity::Warning,
                    help: Some(
                        "Replace `var` with `let` (or `const` if never reassigned)".to_owned(),
                    ),
                    fix: Some(Fix {
                        message: "Replace `var` with `let`".to_owned(),
                        edits: vec![Edit {
                            span: var_span,
                            replacement: "let".to_owned(),
                        }],
                    }),
                    labels: vec![],
                });
            }
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

    #[test]
    fn test_flags_var() {
        let allocator = Allocator::default();
        let source = "var x = 1;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoVar)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should flag var declaration");
            let first = diags.first();
            assert!(
                first.is_some_and(|d| d.fix.is_some()),
                "should provide a fix"
            );
        }
    }

    #[test]
    fn test_allows_let() {
        let allocator = Allocator::default();
        let source = "let x = 1;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoVar)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(diags.is_empty(), "let should not be flagged");
        }
    }

    #[test]
    fn test_allows_const() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoVar)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(diags.is_empty(), "const should not be flagged");
        }
    }

    #[test]
    fn test_fix_replaces_var_with_let() {
        let allocator = Allocator::default();
        let source = "var x = 1;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoVar)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            let first = diags.first();
            let fix = first.and_then(|d| d.fix.as_ref());
            assert!(fix.is_some(), "should have a fix");
            let edit = fix.and_then(|f| f.edits.first());
            assert_eq!(
                edit.map(|e| e.replacement.as_str()),
                Some("let"),
                "fix should replace with let"
            );
        }
    }
}
