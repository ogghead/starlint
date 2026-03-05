//! Rule: `prefer-default-parameters`
//!
//! Prefer default function parameters over manual `||` or `??` assignment.
//! `x = x || defaultValue` inside a function body should use `function(x = defaultValue)`
//! instead.

use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, Expression, LogicalOperator};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `x = x || default` and `x = x ?? default` patterns.
#[derive(Debug)]
pub struct PreferDefaultParameters;

impl NativeRule for PreferDefaultParameters {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-default-parameters".to_owned(),
            description: "Prefer default function parameters over manual `||`/`??` assignment"
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

        // Must be a plain assignment `=`
        if assign.operator != AssignmentOperator::Assign {
            return;
        }

        // Left side must be a simple identifier
        let AssignmentTarget::AssignmentTargetIdentifier(target_id) = &assign.left else {
            return;
        };
        let target_name = target_id.name.as_str();

        // Right side must be a logical expression (`||` or `??`)
        let Expression::LogicalExpression(logical) = &assign.right else {
            return;
        };

        if !matches!(
            logical.operator,
            LogicalOperator::Or | LogicalOperator::Coalesce
        ) {
            return;
        }

        // The left side of the logical expression must be an identifier with the
        // same name as the assignment target (i.e., `x = x || ...`)
        let Expression::Identifier(logical_left) = &logical.left else {
            return;
        };

        if logical_left.name.as_str() != target_name {
            return;
        }

        let operator_str = match logical.operator {
            LogicalOperator::Or => "||",
            LogicalOperator::Coalesce => "??",
            LogicalOperator::And => return,
        };

        ctx.report(Diagnostic {
            rule_name: "prefer-default-parameters".to_owned(),
            message: format!(
                "`{target_name} = {target_name} {operator_str} ...` can be replaced with a default parameter"
            ),
            span: Span::new(assign.span.start, assign.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDefaultParameters)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_or_pattern() {
        let diags = lint("function foo(x) { x = x || 'default'; }");
        assert_eq!(diags.len(), 1, "x = x || 'default' should be flagged");
    }

    #[test]
    fn test_flags_coalesce_pattern() {
        let diags = lint("function foo(x) { x = x ?? 'default'; }");
        assert_eq!(diags.len(), 1, "x = x ?? 'default' should be flagged");
    }

    #[test]
    fn test_allows_default_param() {
        let diags = lint("function foo(x = 'default') { }");
        assert!(
            diags.is_empty(),
            "default parameter syntax should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_variable() {
        let diags = lint("let x = y || 'default';");
        assert!(
            diags.is_empty(),
            "different variable on left should not be flagged"
        );
    }

    #[test]
    fn test_allows_and_operator() {
        let diags = lint("function foo(x) { x = x && 'default'; }");
        assert!(diags.is_empty(), "&& should not be flagged");
    }

    #[test]
    fn test_allows_compound_assignment() {
        let diags = lint("x += x || 1;");
        assert!(
            diags.is_empty(),
            "compound assignment should not be flagged"
        );
    }
}
