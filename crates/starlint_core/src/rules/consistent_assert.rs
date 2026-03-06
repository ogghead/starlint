//! Rule: `consistent-assert`
//!
//! Prefer strict assertion methods over loose ones.
//! Use `assert.strictEqual` instead of `assert.equal`,
//! `assert.notStrictEqual` instead of `assert.notEqual`,
//! `assert.deepStrictEqual` instead of `assert.deepEqual`,
//! and `assert.notDeepStrictEqual` instead of `assert.notDeepEqual`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags loose assertion methods on the `assert` object.
#[derive(Debug)]
pub struct ConsistentAssert;

/// Map from loose method name to its strict equivalent.
fn strict_equivalent(method: &str) -> Option<&'static str> {
    match method {
        "equal" => Some("strictEqual"),
        "notEqual" => Some("notStrictEqual"),
        "deepEqual" => Some("deepStrictEqual"),
        "notDeepEqual" => Some("notDeepStrictEqual"),
        _ => None,
    }
}

impl NativeRule for ConsistentAssert {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-assert".to_owned(),
            description: "Prefer strict assertion methods".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        // Check that the object is `assert`
        let Expression::Identifier(id) = &member.object else {
            return;
        };

        if id.name.as_str() != "assert" {
            return;
        }

        let method = member.property.name.as_str();
        let Some(replacement) = strict_equivalent(method) else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "consistent-assert".to_owned(),
            message: format!("Use `assert.{replacement}()` instead of `assert.{method}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Replace `assert.{method}` with `assert.{replacement}` for strict comparison"
            )),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Replace `{method}` with `{replacement}`"),
                edits: vec![Edit {
                    span: Span::new(member.property.span.start, member.property.span.end),
                    replacement: replacement.to_owned(),
                }],
                is_snippet: false,
            }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentAssert)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_equal() {
        let diags = lint("assert.equal(a, b);");
        assert_eq!(diags.len(), 1, "assert.equal should be flagged");
    }

    #[test]
    fn test_flags_not_equal() {
        let diags = lint("assert.notEqual(a, b);");
        assert_eq!(diags.len(), 1, "assert.notEqual should be flagged");
    }

    #[test]
    fn test_flags_deep_equal() {
        let diags = lint("assert.deepEqual(a, b);");
        assert_eq!(diags.len(), 1, "assert.deepEqual should be flagged");
    }

    #[test]
    fn test_flags_not_deep_equal() {
        let diags = lint("assert.notDeepEqual(a, b);");
        assert_eq!(diags.len(), 1, "assert.notDeepEqual should be flagged");
    }

    #[test]
    fn test_allows_strict_equal() {
        let diags = lint("assert.strictEqual(a, b);");
        assert!(diags.is_empty(), "assert.strictEqual should not be flagged");
    }

    #[test]
    fn test_allows_deep_strict_equal() {
        let diags = lint("assert.deepStrictEqual(a, b);");
        assert!(
            diags.is_empty(),
            "assert.deepStrictEqual should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_assert_object() {
        let diags = lint("foo.equal(a, b);");
        assert!(diags.is_empty(), "non-assert object should not be flagged");
    }

    #[test]
    fn test_allows_other_assert_methods() {
        let diags = lint("assert.ok(a);");
        assert!(diags.is_empty(), "assert.ok should not be flagged");
    }
}
