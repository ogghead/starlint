//! Rule: `no-unmodified-loop-condition`
//!
//! Flag `while`/`do-while` loops where the condition variable is never
//! modified inside the loop body. This is a common source of infinite loops.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags while loops where the condition variable is not modified in the body.
#[derive(Debug)]
pub struct NoUnmodifiedLoopCondition;

/// Extract a simple identifier name from an expression (only handles plain identifiers).
fn extract_test_identifier<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

/// Check if the body source text contains patterns that modify the given identifier.
///
/// Looks for assignment operators, increment, and decrement patterns.
fn body_modifies_identifier(source: &str, body_start: usize, body_end: usize, name: &str) -> bool {
    let Some(body_text) = source.get(body_start..body_end) else {
        return true; // If we can't read the body, assume it might be modified
    };

    // Check for patterns like: name =, name +=, name -=, name++, name--, ++name, --name
    let assignment_pattern = format!("{name} =");
    let plus_assign = format!("{name} +=");
    let minus_assign = format!("{name} -=");
    let times_assign = format!("{name} *=");
    let div_assign = format!("{name} /=");
    let postfix_inc = format!("{name}++");
    let postfix_dec = format!("{name}--");
    let prefix_inc = format!("++{name}");
    let prefix_dec = format!("--{name}");

    body_text.contains(&assignment_pattern)
        || body_text.contains(&plus_assign)
        || body_text.contains(&minus_assign)
        || body_text.contains(&times_assign)
        || body_text.contains(&div_assign)
        || body_text.contains(&postfix_inc)
        || body_text.contains(&postfix_dec)
        || body_text.contains(&prefix_inc)
        || body_text.contains(&prefix_dec)
}

impl NativeRule for NoUnmodifiedLoopCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unmodified-loop-condition".to_owned(),
            description: "Disallow unmodified loop conditions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::WhileStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::WhileStatement(stmt) = kind else {
            return;
        };

        let Some(ident_name) = extract_test_identifier(&stmt.test) else {
            return;
        };

        let body_span = stmt.body.span();
        let body_start = usize::try_from(body_span.start).unwrap_or(0);
        let body_end = usize::try_from(body_span.end).unwrap_or(0);

        if !body_modifies_identifier(ctx.source_text(), body_start, body_end, ident_name) {
            ctx.report(Diagnostic {
                rule_name: "no-unmodified-loop-condition".to_owned(),
                message: format!("`{ident_name}` is not modified in the loop body"),
                span: Span::new(stmt.span.start, stmt.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnmodifiedLoopCondition)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unmodified_condition() {
        let diags = lint("while (x) { doSomething(); }");
        assert_eq!(
            diags.len(),
            1,
            "loop where x is never modified should be flagged"
        );
    }

    #[test]
    fn test_allows_modified_by_assignment() {
        let diags = lint("while (x) { x = false; }");
        assert!(
            diags.is_empty(),
            "loop where x is assigned should not be flagged"
        );
    }

    #[test]
    fn test_allows_modified_by_decrement() {
        let diags = lint("while (x) { x--; }");
        assert!(
            diags.is_empty(),
            "loop where x is decremented should not be flagged"
        );
    }

    #[test]
    fn test_allows_modified_by_increment() {
        let diags = lint("while (x) { x++; }");
        assert!(
            diags.is_empty(),
            "loop where x is incremented should not be flagged"
        );
    }

    #[test]
    fn test_skips_complex_condition() {
        // Complex conditions (not a simple identifier) are skipped
        let diags = lint("while (x > 0) { x--; }");
        assert!(diags.is_empty(), "complex condition should not be checked");
    }
}
