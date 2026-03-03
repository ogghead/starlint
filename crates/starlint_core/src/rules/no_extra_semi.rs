//! Rule: `no-extra-semi`
//!
//! Disallow unnecessary semicolons (empty statements). Extra semicolons are
//! usually the result of a typo or copy-paste error and serve no purpose.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary semicolons (empty statements, e.g. `;;`).
#[derive(Debug)]
pub struct NoExtraSemi;

impl NativeRule for NoExtraSemi {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-extra-semi".to_owned(),
            description: "Disallow unnecessary semicolons".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::EmptyStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if let AstKind::EmptyStatement(stmt) = kind {
            ctx.report(Diagnostic {
                rule_name: "no-extra-semi".to_owned(),
                message: "Unnecessary semicolon".to_owned(),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Warning,
                help: Some("Remove the extra semicolon".to_owned()),
                fix: Some(Fix {
                    message: "Remove the extra semicolon".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(stmt.span.start, stmt.span.end),
                        replacement: String::new(),
                    }],
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

    #[test]
    fn test_flags_extra_semicolon() {
        let allocator = Allocator::default();
        let source = ";;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraSemi)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(!diags.is_empty(), "should flag extra semicolons");
        }
    }

    #[test]
    fn test_allows_necessary_semicolons() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraSemi)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert!(
                diags.is_empty(),
                "necessary semicolons should not be flagged"
            );
        }
    }

    #[test]
    fn test_fix_removes_semicolon() {
        let allocator = Allocator::default();
        let source = ";;";
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExtraSemi)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            let first = diags.first();
            assert!(first.is_some(), "should have at least one diagnostic");
            if let Some(diag) = first {
                let fix = diag.fix.as_ref();
                assert!(fix.is_some(), "diagnostic should have a fix");
                if let Some(f) = fix {
                    let edit = f.edits.first();
                    assert!(edit.is_some(), "fix should have at least one edit");
                    if let Some(e) = edit {
                        assert!(
                            e.replacement.is_empty(),
                            "fix replacement should be empty string"
                        );
                    }
                }
            }
        }
    }
}
