//! Rule: `no-empty-static-block`
//!
//! Disallow empty static initialization blocks. An empty `static {}` block
//! in a class has no effect and is almost certainly a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags empty `static {}` blocks in classes.
#[derive(Debug)]
pub struct NoEmptyStaticBlock;

impl NativeRule for NoEmptyStaticBlock {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-empty-static-block".to_owned(),
            description: "Disallow empty static initialization blocks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::StaticBlock])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::StaticBlock(block) = kind else {
            return;
        };

        if block.body.is_empty() {
            ctx.report_error(
                "no-empty-static-block",
                "Unexpected empty static block",
                Span::new(block.span.start, block.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoEmptyStaticBlock)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_static_block() {
        let diags = lint("class Foo { static {} }");
        assert_eq!(diags.len(), 1, "empty static block should be flagged");
    }

    #[test]
    fn test_allows_non_empty_static_block() {
        let diags = lint("class Foo { static { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "non-empty static block should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_without_static_block() {
        let diags = lint("class Foo { constructor() {} }");
        assert!(
            diags.is_empty(),
            "class without static block should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_empty_static_blocks() {
        let diags = lint("class Foo { static {} static {} }");
        assert_eq!(
            diags.len(),
            2,
            "two empty static blocks should both be flagged"
        );
    }
}
