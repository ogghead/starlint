//! Rule: `promise/no-nesting`
//!
//! Forbid nesting `.then()` or `.catch()` inside another `.then()`/`.catch()`.
//! Nested promise chains flatten poorly and should be refactored to chained
//! `.then()` calls or `async`/`await`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.then()` or `.catch()` calls whose callee object is itself
/// a `.then()` or `.catch()` call inside an argument position, detected
/// by scanning the source text of callback arguments.
#[derive(Debug)]
pub struct NoNesting;

impl NativeRule for NoNesting {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/no-nesting".to_owned(),
            description: "Forbid nesting `.then()`/`.catch()` chains".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();
        if method != "then" && method != "catch" {
            return;
        }

        // Check each argument for nested .then()/.catch() patterns
        for arg in &call.arguments {
            let arg_expr = match arg {
                oxc_ast::ast::Argument::SpreadElement(_) => continue,
                _ => arg.to_expression(),
            };

            let start = usize::try_from(arg_expr.span().start).unwrap_or(0);
            let end = usize::try_from(arg_expr.span().end).unwrap_or(0);
            let body_text = ctx.source_text().get(start..end).unwrap_or_default();

            if body_text.contains(".then(") || body_text.contains(".catch(") {
                ctx.report_warning(
                    "promise/no-nesting",
                    "Avoid nesting `.then()`/`.catch()` — flatten the chain or use `async`/`await`",
                    Span::new(call.span.start, call.span.end),
                );
                return; // Only report once per call
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNesting)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_nested_then() {
        let diags = lint("p.then(val => val.then(x => x));");
        assert!(!diags.is_empty(), "should flag nested .then()");
    }

    #[test]
    fn test_flags_nested_catch_in_then() {
        let diags = lint("p.then(val => other.catch(e => e));");
        assert!(!diags.is_empty(), "should flag nested .catch() in .then()");
    }

    #[test]
    fn test_allows_flat_chain() {
        let diags = lint("p.then(val => val * 2).catch(err => err);");
        assert!(diags.is_empty(), "flat chain should not be flagged");
    }
}
