//! Rule: `no-useless-catch`
//!
//! Disallow catch clauses that only rethrow the caught error.
//! `try { ... } catch (e) { throw e; }` is equivalent to just the try body.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Expression, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags catch clauses that only rethrow without transformation.
#[derive(Debug)]
pub struct NoUselessCatch;

impl NativeRule for NoUselessCatch {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-catch".to_owned(),
            description: "Disallow catch clauses that only rethrow".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TryStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TryStatement(stmt) = kind else {
            return;
        };

        let Some(handler) = &stmt.handler else {
            return;
        };

        // Must have a simple identifier parameter.
        let Some(param) = &handler.param else {
            return;
        };
        let BindingPattern::BindingIdentifier(param_id) = &param.pattern else {
            return;
        };
        let param_name = param_id.name.as_str();

        // Body must have exactly one statement: `throw <same identifier>`.
        if handler.body.body.len() != 1 {
            return;
        }
        let Some(Statement::ThrowStatement(throw_stmt)) = handler.body.body.first() else {
            return;
        };
        let Expression::Identifier(thrown_id) = &throw_stmt.argument else {
            return;
        };
        if thrown_id.name.as_str() != param_name {
            return;
        }

        // This is a useless catch. Build the fix.
        let source = ctx.source_text();

        let (replacement, fix_message) = if stmt.finalizer.is_some() {
            // Has finally — remove the catch clause, keep try + finally.
            // Replace from end of try block to start of finally block.
            let catch_end = handler.span.end;

            // Find the span between try block end and finalizer start that
            // covers the entire catch clause. We replace [catch_start..catch_end]
            // with empty text, but need to remove leading whitespace too.
            // Instead, replace the catch clause span with empty string.
            return ctx.report(Diagnostic {
                rule_name: "no-useless-catch".to_owned(),
                message: "Unnecessary catch clause — only rethrows the error".to_owned(),
                span: Span::new(handler.span.start, handler.span.end),
                severity: Severity::Warning,
                help: Some("Remove the useless catch clause".to_owned()),
                fix: Some(Fix {
                    message: "Remove catch clause".to_owned(),
                    edits: vec![Edit {
                        // Remove from end of try block to start of finally.
                        // We trim catch + surrounding space by replacing
                        // [try_block_end..finalizer_start] with a space.
                        span: Span::new(stmt.block.span.end, catch_end),
                        replacement: String::new(),
                    }],
                }),
                labels: vec![],
            });
        } else {
            // No finally — unwrap the try block entirely.
            let block_start = usize::try_from(stmt.block.span.start).unwrap_or(0);
            let block_end = usize::try_from(stmt.block.span.end).unwrap_or(0);
            let Some(block_text) = source.get(block_start..block_end) else {
                return;
            };

            // Strip the outer braces from the try block: `{ body }` → `body`.
            // Trim leading `{` and trailing `}`, then dedent.
            let inner = block_text
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .unwrap_or(block_text)
                .trim();

            (inner.to_owned(), "Unwrap try block".to_owned())
        };

        ctx.report(Diagnostic {
            rule_name: "no-useless-catch".to_owned(),
            message: "Unnecessary try/catch — catch clause only rethrows the error".to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
            severity: Severity::Warning,
            help: Some("Remove the try/catch wrapper".to_owned()),
            fix: Some(Fix {
                message: fix_message,
                edits: vec![Edit {
                    span: Span::new(stmt.span.start, stmt.span.end),
                    replacement,
                }],
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) else {
            return vec![];
        };
        let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessCatch)];
        traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
    }

    #[test]
    fn test_flags_useless_catch() {
        let diags = lint("try { doSomething(); } catch (e) { throw e; }");
        assert_eq!(diags.len(), 1, "should flag useless catch");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "should provide a fix");
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("doSomething();"),
            "fix should unwrap the try block"
        );
    }

    #[test]
    fn test_flags_useless_catch_with_finally() {
        let diags = lint("try { x(); } catch (e) { throw e; } finally { cleanup(); }");
        assert_eq!(
            diags.len(),
            1,
            "should flag useless catch when finally exists"
        );
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        // Fix removes the catch clause, keeping try + finally.
        assert!(fix.is_some(), "should provide a fix");
    }

    #[test]
    fn test_ignores_catch_with_transformation() {
        let diags = lint("try { x(); } catch (e) { throw new Error(e.message); }");
        assert!(
            diags.is_empty(),
            "catch with error transformation should not be flagged"
        );
    }

    #[test]
    fn test_ignores_catch_with_multiple_statements() {
        let diags = lint("try { x(); } catch (e) { console.log(e); throw e; }");
        assert!(
            diags.is_empty(),
            "catch with multiple statements should not be flagged"
        );
    }

    #[test]
    fn test_ignores_catch_throwing_different_error() {
        let diags = lint("try { x(); } catch (e) { throw otherError; }");
        assert!(
            diags.is_empty(),
            "catch throwing different identifier should not be flagged"
        );
    }

    #[test]
    fn test_ignores_catch_without_param() {
        let diags = lint("try { x(); } catch { throw new Error(); }");
        assert!(
            diags.is_empty(),
            "catch without param should not be flagged"
        );
    }

    #[test]
    fn test_ignores_destructured_catch_param() {
        let diags = lint("try { x(); } catch ({ message }) { throw message; }");
        assert!(
            diags.is_empty(),
            "destructured catch param should not be flagged"
        );
    }
}
