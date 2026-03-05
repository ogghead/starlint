//! Rule: `prefer-dom-node-dataset`
//!
//! Prefer `element.dataset.foo` over `element.getAttribute('data-foo')` or
//! `element.setAttribute('data-foo', value)`. The `dataset` API is more
//! readable and less error-prone.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `getAttribute`/`setAttribute` calls with `data-` prefixed string arguments.
#[derive(Debug)]
pub struct PreferDomNodeDataset;

/// Method names that operate on `data-*` attributes.
const DATA_ATTR_METHODS: &[&str] = &["getAttribute", "setAttribute"];

impl NativeRule for PreferDomNodeDataset {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-dom-node-dataset".to_owned(),
            description: "Prefer `element.dataset` over `getAttribute`/`setAttribute` with `data-*` attributes".to_owned(),
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

        // Callee must be a static member expression like `el.getAttribute(...)`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();

        if !DATA_ATTR_METHODS.contains(&method_name) {
            return;
        }

        // The first argument must be a string literal starting with "data-"
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let first_arg_expr = first_arg.as_expression();

        let Some(Expression::StringLiteral(lit)) = first_arg_expr else {
            return;
        };

        if !lit.value.starts_with("data-") {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-dom-node-dataset".to_owned(),
            message: format!(
                "Prefer `element.dataset` over `{method_name}` with a `data-*` attribute"
            ),
            span: Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferDomNodeDataset)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_get_attribute_data() {
        let diags = lint("el.getAttribute('data-id');");
        assert_eq!(
            diags.len(),
            1,
            "getAttribute with data- attribute should be flagged"
        );
    }

    #[test]
    fn test_flags_set_attribute_data() {
        let diags = lint("el.setAttribute('data-name', 'foo');");
        assert_eq!(
            diags.len(),
            1,
            "setAttribute with data- attribute should be flagged"
        );
    }

    #[test]
    fn test_allows_get_attribute_non_data() {
        let diags = lint("el.getAttribute('class');");
        assert!(
            diags.is_empty(),
            "getAttribute without data- prefix should not be flagged"
        );
    }

    #[test]
    fn test_allows_dataset_access() {
        let diags = lint("el.dataset.id;");
        assert!(diags.is_empty(), "dataset access should not be flagged");
    }

    #[test]
    fn test_allows_set_attribute_non_data() {
        let diags = lint("el.setAttribute('id', 'main');");
        assert!(
            diags.is_empty(),
            "setAttribute without data- prefix should not be flagged"
        );
    }

    #[test]
    fn test_allows_unrelated_call() {
        let diags = lint("el.removeAttribute('data-id');");
        assert!(diags.is_empty(), "removeAttribute should not be flagged");
    }
}
