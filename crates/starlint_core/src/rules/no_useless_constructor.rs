//! Rule: `no-useless-constructor`
//!
//! Disallow unnecessary constructors. An empty constructor or one that simply
//! delegates to `super()` with the same arguments is unnecessary.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    Argument, ClassElement, Expression, MethodDefinitionKind, Statement,
};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags constructors that don't do anything useful.
#[derive(Debug)]
pub struct NoUselessConstructor;

impl NativeRule for NoUselessConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-constructor".to_owned(),
            description: "Disallow unnecessary constructors".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        let has_super = class.super_class.is_some();

        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };

            if method.kind != MethodDefinitionKind::Constructor {
                continue;
            }

            let Some(body) = &method.value.body else {
                continue;
            };

            let params = &method.value.params;

            // Empty constructor with no super class
            if body.statements.is_empty() && !has_super {
                ctx.report_error(
                    "no-useless-constructor",
                    "Useless constructor — empty constructor is unnecessary",
                    Span::new(method.span.start, method.span.end),
                );
                continue;
            }

            // Constructor that only calls super(...args) with same params
            if has_super && body.statements.len() == 1 {
                if let Some(Statement::ExpressionStatement(expr_stmt)) =
                    body.statements.first()
                {
                    if is_simple_super_call(&expr_stmt.expression, params) {
                        ctx.report_error(
                            "no-useless-constructor",
                            "Useless constructor — constructor simply delegates to `super()` with the same arguments",
                            Span::new(method.span.start, method.span.end),
                        );
                    }
                }
            }
        }
    }
}

/// Check if an expression is a `super(...)` call that passes through exactly
/// the same parameters.
fn is_simple_super_call(
    expr: &Expression<'_>,
    params: &oxc_ast::ast::FormalParameters<'_>,
) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    // Must be a super() call
    if !matches!(&call.callee, Expression::Super(_)) {
        return false;
    }

    let param_count = params.items.len();

    // Check if super() is called with the exact same number of arguments
    if call.arguments.len() != param_count && params.rest.is_none() {
        return false;
    }

    // For zero params without rest, super() is a simple passthrough
    if param_count == 0 && params.rest.is_none() && call.arguments.is_empty() {
        return true;
    }

    // Check if each argument is a simple identifier matching the param
    for (arg, param) in call.arguments.iter().zip(params.items.iter()) {
        let Argument::Identifier(arg_id) = arg else {
            return false;
        };
        let oxc_ast::ast::BindingPattern::BindingIdentifier(param_id) = &param.pattern else {
            return false;
        };
        if arg_id.name != param_id.name {
            return false;
        }
    }

    true
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessConstructor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_empty_constructor() {
        let diags = lint("class A { constructor() {} }");
        assert_eq!(
            diags.len(),
            1,
            "empty constructor should be flagged"
        );
    }

    #[test]
    fn test_flags_super_only_constructor() {
        let diags = lint("class B extends A { constructor() { super(); } }");
        assert_eq!(
            diags.len(),
            1,
            "constructor that only calls super() should be flagged"
        );
    }

    #[test]
    fn test_flags_super_passthrough() {
        let diags = lint("class B extends A { constructor(x, y) { super(x, y); } }");
        assert_eq!(
            diags.len(),
            1,
            "constructor that passes through args to super() should be flagged"
        );
    }

    #[test]
    fn test_allows_constructor_with_body() {
        let diags = lint("class A { constructor() { this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "constructor with body should not be flagged"
        );
    }

    #[test]
    fn test_allows_constructor_with_extra_logic() {
        let diags = lint("class B extends A { constructor() { super(); this.x = 1; } }");
        assert!(
            diags.is_empty(),
            "constructor with super + extra logic should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_constructor() {
        let diags = lint("class A { method() {} }");
        assert!(
            diags.is_empty(),
            "class without constructor should not be flagged"
        );
    }
}
