//! Rule: `no-magic-array-flat-depth`
//!
//! Flag `.flat(n)` calls where `n` is a numeric literal greater than 1.
//! Non-trivial flat depths are magic numbers that should be extracted to
//! named constants. `.flat()`, `.flat(1)`, `.flat(Infinity)`, and
//! `.flat(depth)` (variable) are all acceptable.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.flat(n)` calls with magic number depths greater than 1.
#[derive(Debug)]
pub struct NoMagicArrayFlatDepth;

impl NativeRule for NoMagicArrayFlatDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-magic-array-flat-depth".to_owned(),
            description: "Disallow magic number depths in `Array.prototype.flat()` calls"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for `.flat(...)` member call
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "flat" {
            return;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return;
        }

        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Only flag numeric literals > 1
        // Allow: .flat(), .flat(1), .flat(Infinity), .flat(someVar)
        if is_magic_flat_depth(first_arg) {
            ctx.report_warning(
                "no-magic-array-flat-depth",
                "Magic number depth in `.flat()` — use a named constant for non-trivial flat depths",
                Span::new(call.span.start, call.span.end),
            );
        }
    }
}

/// Check if an argument is a numeric literal with value > 1.
///
/// Returns `false` for non-numeric arguments (variables, `Infinity`, etc.).
fn is_magic_flat_depth(arg: &Argument<'_>) -> bool {
    match arg {
        Argument::NumericLiteral(num) => {
            // Allow .flat(1) — this is the default depth
            // Flag .flat(2), .flat(3), .flat(5), etc.
            num.value > 1.0 && num.value.is_finite()
        }
        // .flat(Infinity) is a common pattern — allow it
        // .flat(someVariable) is fine — it's not a magic number
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMagicArrayFlatDepth)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_flat_with_magic_number() {
        let diags = lint("arr.flat(3);");
        assert_eq!(diags.len(), 1, "flat(3) should be flagged as magic number");
    }

    #[test]
    fn test_flags_flat_with_depth_two() {
        let diags = lint("arr.flat(2);");
        assert_eq!(diags.len(), 1, "flat(2) should be flagged as magic number");
    }

    #[test]
    fn test_flags_flat_with_depth_five() {
        let diags = lint("arr.flat(5);");
        assert_eq!(diags.len(), 1, "flat(5) should be flagged as magic number");
    }

    #[test]
    fn test_allows_flat_no_args() {
        let diags = lint("arr.flat();");
        assert!(
            diags.is_empty(),
            "flat() with no args should not be flagged"
        );
    }

    #[test]
    fn test_allows_flat_one() {
        let diags = lint("arr.flat(1);");
        assert!(
            diags.is_empty(),
            "flat(1) should not be flagged (default depth)"
        );
    }

    #[test]
    fn test_allows_flat_infinity() {
        let diags = lint("arr.flat(Infinity);");
        assert!(
            diags.is_empty(),
            "flat(Infinity) should not be flagged (common pattern)"
        );
    }

    #[test]
    fn test_allows_flat_variable() {
        let diags = lint("arr.flat(depth);");
        assert!(
            diags.is_empty(),
            "flat(variable) should not be flagged (not a magic number)"
        );
    }

    #[test]
    fn test_allows_non_flat_call() {
        let diags = lint("arr.map(x => x);");
        assert!(diags.is_empty(), "non-flat calls should not be flagged");
    }
}
