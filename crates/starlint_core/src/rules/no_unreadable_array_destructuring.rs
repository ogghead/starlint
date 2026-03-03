//! Rule: `no-unreadable-array-destructuring` (unicorn)
//!
//! Disallow array destructuring with more than 3 consecutive ignored elements.
//! For example, `const [,,,,val] = arr` is hard to read.

use oxc_ast::AstKind;
use oxc_ast::ast::BindingPattern;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Maximum consecutive holes allowed before flagging.
const MAX_CONSECUTIVE_HOLES: usize = 3;

/// Flags unreadable array destructuring with many consecutive holes.
#[derive(Debug)]
pub struct NoUnreadableArrayDestructuring;

impl NativeRule for NoUnreadableArrayDestructuring {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unreadable-array-destructuring".to_owned(),
            description: "Disallow array destructuring with many consecutive ignored elements"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::VariableDeclarator])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::VariableDeclarator(decl) = kind else {
            return;
        };

        let BindingPattern::ArrayPattern(array_pat) = &decl.id else {
            return;
        };

        // Count consecutive None elements (holes)
        let mut consecutive_holes = 0_usize;
        let mut max_holes = 0_usize;
        for element in &array_pat.elements {
            if element.is_none() {
                consecutive_holes = consecutive_holes.saturating_add(1);
                if consecutive_holes > max_holes {
                    max_holes = consecutive_holes;
                }
            } else {
                consecutive_holes = 0;
            }
        }

        if max_holes > MAX_CONSECUTIVE_HOLES {
            ctx.report_warning(
                "no-unreadable-array-destructuring",
                "Array destructuring with many consecutive ignored elements is hard to read — \
                 use `array[index]` instead",
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnreadableArrayDestructuring)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_many_holes() {
        let diags = lint("const [,,,,val] = arr;");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_few_holes() {
        let diags = lint("const [,,val] = arr;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_normal_destructuring() {
        let diags = lint("const [a, b, c] = arr;");
        assert!(diags.is_empty());
    }
}
