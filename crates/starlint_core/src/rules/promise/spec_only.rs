//! Rule: `promise/spec-only`
//!
//! Forbid non-standard Promise methods. Flags usage of methods that are
//! not part of the ECMAScript specification (e.g. Bluebird extensions).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Standard ECMAScript `Promise` static methods.
const SPEC_STATIC_METHODS: &[&str] = &[
    "resolve",
    "reject",
    "all",
    "allSettled",
    "any",
    "race",
    "withResolvers",
];

/// Standard ECMAScript `Promise` instance methods.
const SPEC_INSTANCE_METHODS: &[&str] = &["then", "catch", "finally"];

/// Flags non-standard Promise static method calls (e.g. `Promise.map`,
/// `Promise.try`, etc.).
#[derive(Debug)]
pub struct SpecOnly;

impl NativeRule for SpecOnly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/spec-only".to_owned(),
            description: "Forbid non-standard Promise methods".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        let method = member.property.name.as_str();

        // Check for non-standard static methods: Promise.xxx()
        if let Expression::Identifier(ident) = &member.object {
            if ident.name.as_str() == "Promise" && !SPEC_STATIC_METHODS.contains(&method) {
                ctx.report(Diagnostic {
                    rule_name: "promise/spec-only".to_owned(),
                    message: format!("`Promise.{method}` is not a standard ECMAScript method"),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Error,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
            }
        }

        // We do not flag instance methods here since we cannot statically
        // determine whether the callee object is a Promise instance.
        // Only flag static methods on the `Promise` global.
        let _ = SPEC_INSTANCE_METHODS;
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SpecOnly)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_promise_map() {
        let diags = lint("Promise.map([1, 2], fn);");
        assert_eq!(diags.len(), 1, "should flag non-standard Promise.map");
    }

    #[test]
    fn test_flags_promise_try() {
        let diags = lint("Promise.try(() => 1);");
        assert_eq!(diags.len(), 1, "should flag non-standard Promise.try");
    }

    #[test]
    fn test_allows_promise_all() {
        let diags = lint("Promise.all([p1, p2]);");
        assert!(diags.is_empty(), "Promise.all is a standard method");
    }
}
