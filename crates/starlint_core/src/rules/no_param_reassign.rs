//! Rule: `no-param-reassign`
//!
//! Disallow reassignment of function parameters. Modifying parameters
//! can lead to confusing behavior and unexpected side effects.
//! This is a simplified version that flags direct assignment to parameter names.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, FormalParameters, Statement};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags reassignment of function parameters (simplified).
#[derive(Debug)]
pub struct NoParamReassign;

impl NativeRule for NoParamReassign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-param-reassign".to_owned(),
            description: "Disallow reassignment of function parameters".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Look for function declarations/expressions and check their body
        // for assignments to parameter names
        let (params, body) = match kind {
            AstKind::Function(func) => {
                let Some(body) = &func.body else {
                    return;
                };
                (&func.params, &body.statements)
            }
            _ => return,
        };

        let param_names = collect_param_names(params);
        if param_names.is_empty() {
            return;
        }

        // Scan body for direct assignments to parameter names
        let source = ctx.source_text();
        let mut spans_to_report: Vec<(String, Span)> = Vec::new();

        for stmt in body {
            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                if let oxc_ast::ast::Expression::AssignmentExpression(assign) =
                    &expr_stmt.expression
                {
                    let target_span = assign.left.span();
                    let start = usize::try_from(target_span.start).unwrap_or(0);
                    let end = usize::try_from(target_span.end).unwrap_or(0);
                    let target_text = source.get(start..end).unwrap_or("");

                    for name in &param_names {
                        if target_text == name.as_str() {
                            spans_to_report.push((
                                name.clone(),
                                Span::new(assign.span.start, assign.span.end),
                            ));
                        }
                    }
                }
            }
        }

        for (name, span) in spans_to_report {
            ctx.report_warning(
                "no-param-reassign",
                &format!("Assignment to function parameter `{name}`"),
                span,
            );
        }
    }
}

use oxc_span::GetSpan;

/// Collect parameter names from formal parameters.
fn collect_param_names(params: &FormalParameters<'_>) -> Vec<String> {
    let mut names = Vec::new();
    for param in &params.items {
        if let BindingPattern::BindingIdentifier(id) = &param.pattern {
            names.push(id.name.to_string());
        }
    }
    names
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoParamReassign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_param_reassign() {
        let diags = lint("function foo(x) { x = 10; }");
        assert_eq!(diags.len(), 1, "parameter reassignment should be flagged");
    }

    #[test]
    fn test_allows_local_variable() {
        let diags = lint("function foo(x) { var y = 10; }");
        assert!(diags.is_empty(), "local variable should not be flagged");
    }

    #[test]
    fn test_allows_no_reassign() {
        let diags = lint("function foo(x) { return x; }");
        assert!(
            diags.is_empty(),
            "using param without reassign should not be flagged"
        );
    }
}
