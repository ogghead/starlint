//! Rule: `no-setter-return`
//!
//! Disallow returning a value from a setter. Setters cannot return a value;
//! any `return <expr>` inside a setter is ignored and indicates a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::{MethodDefinitionKind, PropertyKind, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `return <value>` statements inside setter functions.
#[derive(Debug)]
pub struct NoSetterReturn;

impl NativeRule for NoSetterReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-setter-return".to_owned(),
            description: "Disallow returning a value from a setter".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::MethodDefinition, AstType::ObjectProperty])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::MethodDefinition(method) if method.kind == MethodDefinitionKind::Set => {
                if let Some(body) = &method.value.body {
                    check_statements_for_value_return(&body.statements, ctx);
                }
            }
            AstKind::ObjectProperty(prop) if prop.kind == PropertyKind::Set => {
                if let oxc_ast::ast::Expression::FunctionExpression(func) = &prop.value {
                    if let Some(body) = &func.body {
                        check_statements_for_value_return(&body.statements, ctx);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Walk statements looking for return statements that have a value.
fn check_statements_for_value_return(stmts: &[Statement<'_>], ctx: &mut NativeLintContext<'_>) {
    for stmt in stmts {
        check_statement_for_value_return(stmt, ctx);
    }
}

/// Check a single statement for `return <value>`.
fn check_statement_for_value_return(stmt: &Statement<'_>, ctx: &mut NativeLintContext<'_>) {
    match stmt {
        Statement::ReturnStatement(ret) => {
            if ret.argument.is_some() {
                ctx.report(Diagnostic {
                    rule_name: "no-setter-return".to_owned(),
                    message: "Setter cannot return a value".to_owned(),
                    span: Span::new(ret.span.start, ret.span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }
        Statement::BlockStatement(block) => {
            check_statements_for_value_return(&block.body, ctx);
        }
        Statement::IfStatement(if_stmt) => {
            check_statement_for_value_return(&if_stmt.consequent, ctx);
            if let Some(alt) = &if_stmt.alternate {
                check_statement_for_value_return(alt, ctx);
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSetterReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_setter_return_value() {
        let diags = lint("class Foo { set bar(v) { return v; } }");
        assert_eq!(diags.len(), 1, "setter returning value should be flagged");
    }

    #[test]
    fn test_allows_setter_bare_return() {
        let diags = lint("class Foo { set bar(v) { this.x = v; return; } }");
        assert!(
            diags.is_empty(),
            "bare return in setter should not be flagged"
        );
    }

    #[test]
    fn test_allows_getter_return() {
        let diags = lint("class Foo { get bar() { return 1; } }");
        assert!(
            diags.is_empty(),
            "getter returning value should not be flagged"
        );
    }

    #[test]
    fn test_flags_object_setter_return() {
        let diags = lint("var obj = { set foo(v) { return v; } };");
        assert_eq!(
            diags.len(),
            1,
            "object setter returning value should be flagged"
        );
    }

    #[test]
    fn test_allows_method_return() {
        let diags = lint("class Foo { bar() { return 1; } }");
        assert!(
            diags.is_empty(),
            "normal method return should not be flagged"
        );
    }
}
