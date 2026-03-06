//! Rule: `typescript/prefer-optional-chain`
//!
//! Prefer optional chaining (`foo?.bar`) over short-circuit evaluation
//! (`foo && foo.bar`). The `&&` pattern is verbose and error-prone compared
//! to the optional chaining operator introduced in ES2020.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, LogicalOperator};
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `foo && foo.bar` patterns that can use optional chaining.
#[derive(Debug)]
pub struct PreferOptionalChain;

impl NativeRule for PreferOptionalChain {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-optional-chain".to_owned(),
            description: "Prefer `?.` optional chaining over `&&` short-circuit guards".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::LogicalExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::LogicalExpression(logical) = kind else {
            return;
        };

        if logical.operator != LogicalOperator::And {
            return;
        }

        // Left side must be a plain identifier
        let Expression::Identifier(left_id) = &logical.left else {
            return;
        };
        let guard_name = left_id.name.as_str();

        // Right side must be a member expression or call on that same identifier
        if !is_member_or_call_on(guard_name, &logical.right, ctx.source_text()) {
            return;
        }

        // Build fix: replace `foo && foo.bar` with `foo?.bar`
        // by inserting `?` after the first `.` in the right side
        let right_span = logical.right.span();
        #[allow(clippy::as_conversions)]
        let fix = ctx
            .source_text()
            .get(right_span.start as usize..right_span.end as usize)
            .and_then(|right_text| {
                // Replace "foo." with "foo?." at the start of the right expression
                let prefix = format!("{guard_name}.");
                right_text.starts_with(prefix.as_str()).then(|| {
                    let replacement = format!("{guard_name}?.{}", &right_text[prefix.len()..]);
                    Fix {
                        message: format!("Replace with `{replacement}`"),
                        edits: vec![Edit {
                            span: Span::new(logical.span.start, logical.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                })
            });

        ctx.report(Diagnostic {
            rule_name: "typescript/prefer-optional-chain".to_owned(),
            message: format!(
                "Prefer `{guard_name}?.` optional chaining over `{guard_name} && {guard_name}.…`"
            ),
            span: Span::new(logical.span.start, logical.span.end),
            severity: Severity::Warning,
            help: Some("Use optional chaining operator `?.`".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

/// Check if the expression is a member access or call on the given identifier.
///
/// Matches patterns like `foo.bar`, `foo.bar()`, `foo["bar"]`, or `foo.bar.baz`.
fn is_member_or_call_on(name: &str, expr: &Expression<'_>, source: &str) -> bool {
    match expr {
        Expression::StaticMemberExpression(member) => {
            object_matches_name(name, &member.object, source)
        }
        Expression::ComputedMemberExpression(member) => {
            object_matches_name(name, &member.object, source)
        }
        Expression::CallExpression(call) => {
            // Check if callee is a member expression on the same identifier
            // e.g. `foo.bar()` where callee is `foo.bar`
            match &call.callee {
                Expression::StaticMemberExpression(member) => {
                    object_matches_name(name, &member.object, source)
                }
                Expression::ComputedMemberExpression(member) => {
                    object_matches_name(name, &member.object, source)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Check if the object of a member expression is the given identifier name.
///
/// Handles both direct identifier (`foo.bar`) and chained member expressions
/// (`foo.bar.baz` by checking the root object).
fn object_matches_name(name: &str, object: &Expression<'_>, source: &str) -> bool {
    match object {
        Expression::Identifier(id) => id.name.as_str() == name,
        // For chained access like `foo.bar.baz`, check if the source starts with the name
        Expression::StaticMemberExpression(member) => {
            object_matches_name(name, &member.object, source)
        }
        Expression::ComputedMemberExpression(member) => {
            let start = usize::try_from(member.object.span().start).unwrap_or(0);
            let end = usize::try_from(member.object.span().end).unwrap_or(0);
            source.get(start..end).is_some_and(|s| s == name)
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferOptionalChain)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_and_member_access() {
        let diags = lint("declare const foo: any; foo && foo.bar;");
        assert_eq!(diags.len(), 1, "`foo && foo.bar` should be flagged");
    }

    #[test]
    fn test_flags_and_method_call() {
        let diags = lint("declare const foo: any; foo && foo.baz();");
        assert_eq!(diags.len(), 1, "`foo && foo.baz()` should be flagged");
    }

    #[test]
    fn test_allows_optional_chaining() {
        let diags = lint("declare const foo: any; foo?.bar;");
        assert!(diags.is_empty(), "`foo?.bar` should not be flagged");
    }

    #[test]
    fn test_allows_different_identifiers() {
        let diags = lint("declare const foo: any; declare const bar: any; foo && bar.baz;");
        assert!(
            diags.is_empty(),
            "`foo && bar.baz` should not be flagged (different identifiers)"
        );
    }

    #[test]
    fn test_allows_or_operator() {
        let diags = lint("declare const foo: any; foo || foo.bar;");
        assert!(diags.is_empty(), "`||` operator should not be flagged");
    }

    #[test]
    fn test_allows_non_member_right() {
        let diags = lint("declare const foo: any; foo && true;");
        assert!(diags.is_empty(), "`foo && true` should not be flagged");
    }
}
