//! Rule: `prefer-prototype-methods`
//!
//! Prefer using prototype methods directly instead of creating temporary
//! instances. Patterns like `[].forEach.call(obj, fn)` or
//! `"".trim.call(str)` create a throwaway literal just to access a
//! prototype method. Use `Array.prototype.forEach.call(obj, fn)` instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.call()`/`.apply()` on methods accessed from empty array or string
/// literals.
#[derive(Debug)]
pub struct PreferPrototypeMethods;

impl NativeRule for PreferPrototypeMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-prototype-methods".to_owned(),
            description: "Prefer prototype methods over creating instances to access them"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Callee must be `<something>.call(...)` or `<something>.apply(...)`
        let Expression::StaticMemberExpression(outer_member) = &call.callee else {
            return;
        };

        let method = outer_member.property.name.as_str();
        if method != "call" && method != "apply" {
            return;
        }

        // The object of the outer `.call()`/`.apply()` must itself be a
        // member expression: `[].forEach` or `"".trim`
        let Expression::StaticMemberExpression(inner_member) = &outer_member.object else {
            return;
        };

        // The innermost object must be an empty array literal or empty string
        // literal.
        let is_empty_array = matches!(
            &inner_member.object,
            Expression::ArrayExpression(arr) if arr.elements.is_empty()
        );

        let is_empty_string = matches!(
            &inner_member.object,
            Expression::StringLiteral(s) if s.value.is_empty()
        );

        if !is_empty_array && !is_empty_string {
            return;
        }

        let prototype_method = inner_member.property.name.as_str();
        let prototype_owner = if is_empty_array { "Array" } else { "String" };

        // Replace the literal (`[]` or `""`) with `Type.prototype`
        let literal_span = inner_member.object.span();
        let replacement = format!("{prototype_owner}.prototype");

        ctx.report(Diagnostic {
            rule_name: "prefer-prototype-methods".to_owned(),
            message: format!(
                "Use `{prototype_owner}.prototype.{prototype_method}.{method}()` instead of a \
                 literal"
            ),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Replace with `{prototype_owner}.prototype.{prototype_method}.{method}()`"
            )),
            fix: Some(Fix {
                message: format!(
                    "Replace with `{prototype_owner}.prototype.{prototype_method}.{method}()`"
                ),
                edits: vec![Edit {
                    span: Span::new(literal_span.start, literal_span.end),
                    replacement,
                }],
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferPrototypeMethods)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_array_foreach_call() {
        let diags = lint("[].forEach.call(obj, fn);");
        assert_eq!(diags.len(), 1, "[].forEach.call() should be flagged");
    }

    #[test]
    fn test_flags_array_map_call() {
        let diags = lint("[].map.call(obj, fn);");
        assert_eq!(diags.len(), 1, "[].map.call() should be flagged");
    }

    #[test]
    fn test_flags_string_trim_call() {
        let diags = lint(r#""".trim.call(str);"#);
        assert_eq!(diags.len(), 1, r#""".trim.call() should be flagged"#);
    }

    #[test]
    fn test_allows_prototype_call() {
        let diags = lint("Array.prototype.forEach.call(obj, fn);");
        assert!(
            diags.is_empty(),
            "Array.prototype.forEach.call() should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_method_call() {
        let diags = lint("arr.forEach(fn);");
        assert!(diags.is_empty(), "normal method call should not be flagged");
    }

    #[test]
    fn test_allows_non_empty_array() {
        let diags = lint("[1].forEach.call(obj, fn);");
        assert!(
            diags.is_empty(),
            "non-empty array literal should not be flagged"
        );
    }

    #[test]
    fn test_flags_array_apply() {
        let diags = lint("[].slice.apply(obj, [1, 2]);");
        assert_eq!(diags.len(), 1, "[].slice.apply() should be flagged");
    }
}
