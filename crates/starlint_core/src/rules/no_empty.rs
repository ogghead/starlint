//! Rule: `no-empty`
//!
//! Disallow empty block statements. Empty blocks are usually the result of
//! incomplete refactoring.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags empty block statements (e.g. `if (x) {}`).
#[derive(Debug)]
pub struct NoEmpty;

impl NativeRule for NoEmpty {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty".to_owned(),
            description: "Disallow empty block statements".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BlockStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::BlockStatement(block) = kind {
            if block.body.is_empty() {
                let span = Span::new(block.span.start, block.span.end);
                ctx.report(Diagnostic {
                    rule_name: "no-empty".to_owned(),
                    message: "Empty block statement".to_owned(),
                    span,
                    severity: Severity::Warning,
                    help: Some("Add a comment inside the block if intentionally empty".to_owned()),
                    fix: Some(Fix {
                        message: "Add `/* empty */` comment".to_owned(),
                        edits: vec![Edit {
                            span,
                            replacement: "{ /* empty */ }".to_owned(),
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
    fn test_flags_empty_block() {
        let allocator = Allocator::default();
        let source = "if (true) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmpty)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should find one empty block");
            assert_eq!(
                diags.first().map(|d| d.rule_name.as_str()),
                Some("no-empty"),
                "rule name should match"
            );
        }
    }

    #[test]
    fn test_ignores_non_empty_block() {
        let allocator = Allocator::default();
        let source = "if (true) { console.log('hi'); }";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmpty)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "non-empty block should have no diagnostics"
            );
        }
    }

    #[test]
    fn test_fix_adds_empty_comment() {
        let allocator = Allocator::default();
        let source = "if (true) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmpty)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1);
            let fix = diags.first().and_then(|d| d.fix.as_ref());
            assert!(fix.is_some(), "should provide a fix");
            let edit = fix.and_then(|f| f.edits.first());
            assert_eq!(
                edit.map(|e| e.replacement.as_str()),
                Some("{ /* empty */ }"),
                "fix should replace block with comment"
            );
        }
    }

    #[test]
    fn test_flags_empty_try_catch() {
        let allocator = Allocator::default();
        let source = "try { doSomething(); } catch (e) {}";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmpty)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "empty catch block should be flagged");
        }
    }
}
