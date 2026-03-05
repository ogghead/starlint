//! Rule: `no-accumulating-spread` (OXC)
//!
//! Detect spread operators used inside loops which create O(n^2) behavior.
//! For example, `result = [...result, item]` inside a loop copies the entire
//! array on each iteration. Use `push()` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags spread in array expressions inside assignments (potential loop accumulation).
#[derive(Debug)]
pub struct NoAccumulatingSpread;

impl NativeRule for NoAccumulatingSpread {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-accumulating-spread".to_owned(),
            description: "Detect spread operators that accumulate in loops (O(n^2))".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // We look for assignments like `x = [...x, item]` or `x = {...x, key: val}`
        // These are O(n^2) when inside loops, but we flag them regardless as a warning
        // since they're almost always better written with push() or Object.assign().
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        let Expression::ArrayExpression(array) = &assign.right else {
            return;
        };

        // Get the target name
        let target_name = match &assign.left {
            oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                Some(id.name.as_str())
            }
            _ => None,
        };

        let Some(target) = target_name else {
            return;
        };

        // Check if the array expression contains a spread of the same variable
        for element in &array.elements {
            let oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) = element else {
                continue;
            };
            if let Expression::Identifier(id) = &spread.argument {
                if id.name.as_str() == target {
                    ctx.report(Diagnostic {
                        rule_name: "no-accumulating-spread".to_owned(),
                        message: format!(
                            "`{target} = [...{target}, ...]` copies the entire array — \
                             use `{target}.push()` instead for better performance"
                        ),
                        span: Span::new(assign.span.start, assign.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                    return;
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
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAccumulatingSpread)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_accumulating_spread() {
        let diags = lint("result = [...result, item];");
        assert_eq!(diags.len(), 1, "accumulating spread should be flagged");
    }

    #[test]
    fn test_flags_prepend_spread() {
        let diags = lint("result = [item, ...result];");
        assert_eq!(diags.len(), 1, "prepend spread should also be flagged");
    }

    #[test]
    fn test_allows_spread_of_different_variable() {
        let diags = lint("result = [...other, item];");
        assert!(
            diags.is_empty(),
            "spread of different variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_push() {
        let diags = lint("result.push(item);");
        assert!(diags.is_empty(), "push should not be flagged");
    }
}
