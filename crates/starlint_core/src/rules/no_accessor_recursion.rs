//! Rule: `no-accessor-recursion` (unicorn)
//!
//! Disallow recursive getters and setters. A getter that accesses its own
//! property or a setter that assigns to its own property causes infinite
//! recursion.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, MethodDefinitionKind, PropertyKey, Statement};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags getters/setters that recursively access/assign their own property.
#[derive(Debug)]
pub struct NoAccessorRecursion;

impl NativeRule for NoAccessorRecursion {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-accessor-recursion".to_owned(),
            description: "Disallow recursive getters and setters".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::MethodDefinition(method) = kind else {
            return;
        };

        let prop_name = match &method.key {
            PropertyKey::StaticIdentifier(id) => id.name.as_str(),
            _ => return,
        };

        match method.kind {
            MethodDefinitionKind::Get => {
                // Check if the getter body accesses `this.propName`
                let Some(func) = &method.value.body else {
                    return;
                };

                for stmt in &func.statements {
                    if statement_accesses_this_property(stmt, prop_name) {
                        ctx.report_error(
                            "no-accessor-recursion",
                            &format!(
                                "Getter for '{prop_name}' recursively accesses `this.{prop_name}`"
                            ),
                            Span::new(method.span.start, method.span.end),
                        );
                        return;
                    }
                }
            }
            MethodDefinitionKind::Set => {
                // Check if the setter body assigns to `this.propName`
                let Some(func) = &method.value.body else {
                    return;
                };

                for stmt in &func.statements {
                    if statement_assigns_this_property(stmt, prop_name) {
                        ctx.report_error(
                            "no-accessor-recursion",
                            &format!(
                                "Setter for '{prop_name}' recursively assigns to `this.{prop_name}`"
                            ),
                            Span::new(method.span.start, method.span.end),
                        );
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if a statement reads `this.propName`.
fn statement_accesses_this_property(stmt: &Statement<'_>, prop_name: &str) -> bool {
    match stmt {
        Statement::ReturnStatement(ret) => ret
            .argument
            .as_ref()
            .is_some_and(|expr| expression_accesses_this_property(expr, prop_name)),
        Statement::ExpressionStatement(expr_stmt) => {
            expression_accesses_this_property(&expr_stmt.expression, prop_name)
        }
        _ => false,
    }
}

/// Check if an expression reads `this.propName`.
fn expression_accesses_this_property(expr: &Expression<'_>, prop_name: &str) -> bool {
    match expr {
        Expression::StaticMemberExpression(member) => {
            matches!(&member.object, Expression::ThisExpression(_))
                && member.property.name == prop_name
        }
        _ => false,
    }
}

/// Check if a statement assigns to `this.propName`.
fn statement_assigns_this_property(stmt: &Statement<'_>, prop_name: &str) -> bool {
    if let Statement::ExpressionStatement(expr_stmt) = stmt {
        if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
            if let oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) = &assign.left {
                return matches!(&member.object, Expression::ThisExpression(_))
                    && member.property.name == prop_name;
            }
        }
    }
    false
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAccessorRecursion)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_recursive_getter() {
        let diags = lint("class Foo { get bar() { return this.bar; } }");
        assert_eq!(diags.len(), 1, "recursive getter should be flagged");
    }

    #[test]
    fn test_flags_recursive_setter() {
        let diags = lint("class Foo { set bar(val) { this.bar = val; } }");
        assert_eq!(diags.len(), 1, "recursive setter should be flagged");
    }

    #[test]
    fn test_allows_non_recursive_getter() {
        let diags = lint("class Foo { get bar() { return this._bar; } }");
        assert!(
            diags.is_empty(),
            "non-recursive getter should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_recursive_setter() {
        let diags = lint("class Foo { set bar(val) { this._bar = val; } }");
        assert!(
            diags.is_empty(),
            "non-recursive setter should not be flagged"
        );
    }
}
