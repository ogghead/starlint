//! Rule: `getter-return`
//!
//! Enforce `return` statements in getters. A getter without a `return`
//! statement implicitly returns `undefined`, which is almost always a bug.

use oxc_ast::AstKind;
use oxc_ast::ast::{MethodDefinitionKind, PropertyKind, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags getters that don't contain a return statement.
#[derive(Debug)]
pub struct GetterReturn;

impl NativeRule for GetterReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "getter-return".to_owned(),
            description: "Enforce `return` statements in getters".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::MethodDefinition, AstType::ObjectProperty])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::MethodDefinition(method) if method.kind == MethodDefinitionKind::Get => {
                if let Some(body) = &method.value.body {
                    if !statements_contain_return(&body.statements) {
                        ctx.report_error(
                            "getter-return",
                            "Expected a return value in getter",
                            Span::new(method.span.start, method.span.end),
                        );
                    }
                }
            }
            AstKind::ObjectProperty(prop) if prop.kind == PropertyKind::Get => {
                if let oxc_ast::ast::Expression::FunctionExpression(func) = &prop.value {
                    if let Some(body) = &func.body {
                        if !statements_contain_return(&body.statements) {
                            ctx.report_error(
                                "getter-return",
                                "Expected a return value in getter",
                                Span::new(prop.span.start, prop.span.end),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if any statement in the list contains a return statement with a value.
fn statements_contain_return(stmts: &[Statement<'_>]) -> bool {
    stmts.iter().any(|s| statement_contains_return(s))
}

/// Recursively check a single statement for a return with a value.
fn statement_contains_return(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(ret) => ret.argument.is_some(),
        Statement::BlockStatement(block) => statements_contain_return(&block.body),
        Statement::IfStatement(if_stmt) => {
            statement_contains_return(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_ref()
                    .is_some_and(|alt| statement_contains_return(alt))
        }
        Statement::SwitchStatement(switch) => switch
            .cases
            .iter()
            .any(|case| case.consequent.iter().any(|s| statement_contains_return(s))),
        Statement::TryStatement(try_stmt) => {
            statements_contain_return(&try_stmt.block.body)
                || try_stmt
                    .handler
                    .as_ref()
                    .is_some_and(|h| statements_contain_return(&h.body.body))
        }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(GetterReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_getter_without_return() {
        let diags = lint("class Foo { get bar() { console.log('hi'); } }");
        assert_eq!(diags.len(), 1, "getter without return should be flagged");
    }

    #[test]
    fn test_allows_getter_with_return() {
        let diags = lint("class Foo { get bar() { return 1; } }");
        assert!(diags.is_empty(), "getter with return should not be flagged");
    }

    #[test]
    fn test_flags_object_getter_without_return() {
        let diags = lint("var obj = { get foo() { console.log('hi'); } };");
        assert_eq!(
            diags.len(),
            1,
            "object getter without return should be flagged"
        );
    }

    #[test]
    fn test_allows_object_getter_with_return() {
        let diags = lint("var obj = { get foo() { return 1; } };");
        assert!(
            diags.is_empty(),
            "object getter with return should not be flagged"
        );
    }

    #[test]
    fn test_allows_return_in_if() {
        let diags = lint("class Foo { get bar() { if (true) { return 1; } } }");
        assert!(
            diags.is_empty(),
            "getter with return in if should not be flagged"
        );
    }

    #[test]
    fn test_allows_setter_without_return() {
        let diags = lint("class Foo { set bar(v) { this.x = v; } }");
        assert!(
            diags.is_empty(),
            "setter without return should not be flagged"
        );
    }
}
