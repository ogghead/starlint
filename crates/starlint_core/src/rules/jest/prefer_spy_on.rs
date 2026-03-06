//! Rule: `jest/prefer-spy-on`
//!
//! Suggest `jest.spyOn(obj, 'method')` over `obj.method = jest.fn()`.
//! `spyOn` preserves the original implementation and can be easily restored,
//! while direct assignment loses the original function reference.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `obj.method = jest.fn()` patterns.
#[derive(Debug)]
pub struct PreferSpyOn;

impl NativeRule for PreferSpyOn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-spy-on".to_owned(),
            description: "Suggest using `jest.spyOn()` instead of `obj.method = jest.fn()`"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Left side must be a member expression (obj.method or obj['method'])
        let is_member_target = matches!(
            &assign.left,
            oxc_ast::ast::AssignmentTarget::StaticMemberExpression(_)
                | oxc_ast::ast::AssignmentTarget::ComputedMemberExpression(_)
        );
        if !is_member_target {
            return;
        }

        // Right side must be `jest.fn()` call
        if !is_jest_fn_call(&assign.right) {
            return;
        }

        // Build fix for StaticMemberExpression targets: `obj.method = jest.fn()` → `jest.spyOn(obj, 'method')`
        let fix =
            if let oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) = &assign.left {
                let source = ctx.source_text();
                #[allow(clippy::as_conversions)]
                let obj_text = source
                    .get(member.object.span().start as usize..member.object.span().end as usize)
                    .unwrap_or("");
                let prop_name = member.property.name.as_str();
                let replacement = format!("jest.spyOn({obj_text}, '{prop_name}')");
                Some(Fix {
                    message: format!("Replace with `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(assign.span.start, assign.span.end),
                        replacement,
                    }],
                })
            } else {
                None
            };

        ctx.report(Diagnostic {
            rule_name: "jest/prefer-spy-on".to_owned(),
            message:
                "Use `jest.spyOn(obj, 'method')` instead of assigning `jest.fn()` to a property"
                    .to_owned(),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Warning,
            help: None,
            fix,
            labels: vec![],
        });
    }
}

/// Check if an expression is a `jest.fn()` call.
fn is_jest_fn_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    let Expression::Identifier(obj) = &member.object else {
        return false;
    };
    obj.name.as_str() == "jest" && member.property.name.as_str() == "fn"
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferSpyOn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_property_assign_jest_fn() {
        let diags = lint("obj.method = jest.fn();");
        assert_eq!(diags.len(), 1, "`obj.method = jest.fn()` should be flagged");
    }

    #[test]
    fn test_allows_spy_on() {
        let diags = lint("jest.spyOn(obj, 'method');");
        assert!(diags.is_empty(), "`jest.spyOn()` should not be flagged");
    }

    #[test]
    fn test_allows_regular_assignment() {
        let diags = lint("obj.method = function() {};");
        assert!(
            diags.is_empty(),
            "regular function assignment should not be flagged"
        );
    }
}
