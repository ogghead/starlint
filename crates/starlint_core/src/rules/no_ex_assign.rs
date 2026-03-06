//! Rule: `no-ex-assign`
//!
//! Disallow reassigning exceptions in `catch` clauses. Overwriting the
//! caught exception destroys the original error information and is almost
//! always a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentTarget, BindingPattern, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags assignments to the catch clause parameter.
#[derive(Debug)]
pub struct NoExAssign;

impl NativeRule for NoExAssign {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-ex-assign".to_owned(),
            description: "Disallow reassigning exceptions in catch clauses".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CatchClause])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CatchClause(catch) = kind else {
            return;
        };

        // Get the name of the catch parameter
        let Some(param) = &catch.param else {
            return;
        };

        let BindingPattern::BindingIdentifier(ident) = &param.pattern else {
            return;
        };

        let param_name = ident.name.as_str();

        // Scan the catch body for assignments to the parameter name
        scan_statements_for_assignment(&catch.body.body, param_name, ctx);
    }
}

/// Recursively scan statements looking for assignments to a given identifier.
fn scan_statements_for_assignment(
    stmts: &[Statement<'_>],
    name: &str,
    ctx: &mut NativeLintContext<'_>,
) {
    for stmt in stmts {
        scan_statement_for_assignment(stmt, name, ctx);
    }
}

/// Check a single statement for assignments to the named identifier.
fn scan_statement_for_assignment(
    stmt: &Statement<'_>,
    name: &str,
    ctx: &mut NativeLintContext<'_>,
) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            if let oxc_ast::ast::Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                if let AssignmentTarget::AssignmentTargetIdentifier(target_ident) = &assign.left {
                    if target_ident.name.as_str() == name {
                        ctx.report(Diagnostic {
                            rule_name: "no-ex-assign".to_owned(),
                            message: format!("Do not assign to the exception parameter `{name}`"),
                            span: Span::new(assign.span.start, assign.span.end),
                            severity: Severity::Error,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
        }
        Statement::BlockStatement(block) => {
            scan_statements_for_assignment(&block.body, name, ctx);
        }
        Statement::IfStatement(if_stmt) => {
            scan_statement_for_assignment(&if_stmt.consequent, name, ctx);
            if let Some(alt) = &if_stmt.alternate {
                scan_statement_for_assignment(alt, name, ctx);
            }
        }
        _ => {}
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoExAssign)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_catch_param_reassign() {
        let diags = lint("try {} catch (e) { e = 10; }");
        assert_eq!(diags.len(), 1, "reassigning catch param should be flagged");
    }

    #[test]
    fn test_allows_catch_param_usage() {
        let diags = lint("try {} catch (e) { console.log(e); }");
        assert!(diags.is_empty(), "using catch param should not be flagged");
    }

    #[test]
    fn test_allows_different_variable_assignment() {
        let diags = lint("try {} catch (e) { let x = 10; }");
        assert!(
            diags.is_empty(),
            "assigning to different variable should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_catch_param() {
        let diags = lint("try {} catch { let x = 1; }");
        assert!(
            diags.is_empty(),
            "catch without param should not be flagged"
        );
    }

    #[test]
    fn test_flags_nested_reassign() {
        let diags = lint("try {} catch (e) { if (true) { e = 10; } }");
        assert_eq!(
            diags.len(),
            1,
            "nested reassign of catch param should be flagged"
        );
    }
}
