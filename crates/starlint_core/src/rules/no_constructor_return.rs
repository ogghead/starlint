//! Rule: `no-constructor-return`
//!
//! Disallow returning a value from a constructor. Constructors should not use
//! `return <value>` — it interferes with the normal `new` operator behavior.
//! A bare `return;` is acceptable for early exit.

use oxc_ast::AstKind;
use oxc_ast::ast::{MethodDefinitionKind, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `return <value>` statements inside class constructors.
#[derive(Debug)]
pub struct NoConstructorReturn;

impl NativeRule for NoConstructorReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constructor-return".to_owned(),
            description: "Disallow returning a value from a constructor".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::MethodDefinition])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::MethodDefinition(method) = kind else {
            return;
        };

        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        let Some(body) = &method.value.body else {
            return;
        };

        check_statements_for_value_return(&body.statements, ctx);
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
                    rule_name: "no-constructor-return".to_owned(),
                    message: "Unexpected return statement in constructor".to_owned(),
                    span: Span::new(ret.span.start, ret.span.end),
                    severity: Severity::Error,
                    help: Some("Remove the return value or use a bare `return;`".to_owned()),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Remove the return value".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(ret.span.start, ret.span.end),
                            replacement: "return;".to_owned(),
                        }],
                        is_snippet: false,
                    }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConstructorReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_return_value_in_constructor() {
        let diags = lint("class Foo { constructor() { return {}; } }");
        assert_eq!(
            diags.len(),
            1,
            "return value in constructor should be flagged"
        );
    }

    #[test]
    fn test_allows_bare_return() {
        let diags = lint("class Foo { constructor() { return; } }");
        assert!(
            diags.is_empty(),
            "bare return in constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_return() {
        let diags = lint("class Foo { constructor() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "constructor without return should not be flagged"
        );
    }

    #[test]
    fn test_allows_method_return() {
        let diags = lint("class Foo { bar() { return 1; } }");
        assert!(diags.is_empty(), "return in method should not be flagged");
    }

    #[test]
    fn test_flags_nested_return() {
        let diags = lint("class Foo { constructor() { if (true) { return 1; } } }");
        assert_eq!(
            diags.len(),
            1,
            "nested return value in constructor should be flagged"
        );
    }
}
