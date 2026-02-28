//! Rule: `typescript/prefer-for-of`
//!
//! Prefer `for...of` loops over index-based `for` loops when the loop counter
//! is only used to access array elements. A `for (let i = 0; i < arr.length; i++)`
//! pattern can often be replaced with the cleaner `for (const item of arr)`.
//!
//! This rule flags `ForStatement` nodes that follow the classic indexed loop
//! pattern: numeric initializer at `0`, a `.length` comparison test, and
//! an increment (`++`) update.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    BinaryOperator, BindingPattern, Expression, ForStatementInit, SimpleAssignmentTarget,
    UpdateOperator,
};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags classic index-based `for` loops that could use `for...of`.
#[derive(Debug)]
pub struct PreferForOf;

impl NativeRule for PreferForOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-for-of".to_owned(),
            description: "Prefer `for...of` loops over index-based `for` loops".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ForStatement(stmt) = kind else {
            return;
        };

        // Step 1: init must be `let i = 0` (variable declaration with numeric literal 0)
        let Some(init) = &stmt.init else {
            return;
        };
        let Some(counter_name) = extract_zero_init(init) else {
            return;
        };

        // Step 2: test must be `i < <something>.length`
        let Some(test) = &stmt.test else {
            return;
        };
        if !is_length_comparison(test, counter_name) {
            return;
        }

        // Step 3: update must be `i++` or `++i`
        let Some(update) = &stmt.update else {
            return;
        };
        if !is_increment(update, counter_name) {
            return;
        }

        ctx.report_warning(
            "typescript/prefer-for-of",
            "This index-based `for` loop can be replaced with a `for...of` loop",
            Span::new(stmt.span.start, stmt.span.end),
        );
    }
}

/// Check if the `for` loop init is a `VariableDeclaration` with a single
/// declarator initialized to `0`. Returns the counter variable name if so.
fn extract_zero_init<'a>(init: &'a ForStatementInit<'a>) -> Option<&'a str> {
    let ForStatementInit::VariableDeclaration(decl) = init else {
        return None;
    };

    let declarator = decl.declarations.first()?;

    // Init must be `0`
    let Some(Expression::NumericLiteral(lit)) = &declarator.init else {
        return None;
    };

    #[allow(clippy::float_cmp)]
    if lit.value != 0.0 {
        return None;
    }

    // Binding must be a simple identifier
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return None;
    };

    Some(ident.name.as_str())
}

/// Check if the test expression is `counter < something.length`.
fn is_length_comparison(test: &Expression<'_>, counter_name: &str) -> bool {
    let Expression::BinaryExpression(bin) = test else {
        return false;
    };

    if bin.operator != BinaryOperator::LessThan {
        return false;
    }

    // Left side must be our counter variable
    let Expression::Identifier(left_id) = &bin.left else {
        return false;
    };
    if left_id.name.as_str() != counter_name {
        return false;
    }

    // Right side must be `<something>.length`
    let Expression::StaticMemberExpression(member) = &bin.right else {
        return false;
    };

    member.property.name.as_str() == "length"
}

/// Check if the update expression is `counter++` or `++counter`.
fn is_increment(update: &Expression<'_>, counter_name: &str) -> bool {
    let Expression::UpdateExpression(upd) = update else {
        return false;
    };

    if upd.operator != UpdateOperator::Increment {
        return false;
    }

    let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &upd.argument else {
        return false;
    };

    ident.name.as_str() == counter_name
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferForOf)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_classic_indexed_loop() {
        let diags = lint("const arr = [1]; for (let i = 0; i < arr.length; i++) { arr[i]; }");
        assert_eq!(
            diags.len(),
            1,
            "classic indexed for loop should be flagged"
        );
    }

    #[test]
    fn test_allows_for_of() {
        let diags = lint("const arr = [1]; for (const x of arr) {}");
        assert!(diags.is_empty(), "`for...of` should not be flagged");
    }

    #[test]
    fn test_allows_non_length_bound() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert!(
            diags.is_empty(),
            "loop with numeric bound (not `.length`) should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_zero_init() {
        let diags = lint("const arr = [1]; for (let i = 1; i < arr.length; i++) {}");
        assert!(
            diags.is_empty(),
            "loop starting at 1 should not be flagged"
        );
    }

    #[test]
    fn test_allows_decrement_update() {
        let diags = lint("const arr = [1]; for (let i = 0; i < arr.length; i--) {}");
        assert!(
            diags.is_empty(),
            "loop with decrement update should not be flagged"
        );
    }

    #[test]
    fn test_flags_prefix_increment() {
        let diags = lint("const arr = [1]; for (let i = 0; i < arr.length; ++i) { arr[i]; }");
        assert_eq!(
            diags.len(),
            1,
            "prefix increment should also be flagged"
        );
    }
}
