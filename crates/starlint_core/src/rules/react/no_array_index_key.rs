//! Rule: `react/no-array-index-key`
//!
//! Warn when an array index is used as the `key` prop in a `.map()` call.
//! Using index as key can cause issues with component state when the list is
//! reordered.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, BindingPattern, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of array index as the `key` prop inside `.map()` callbacks.
#[derive(Debug)]
pub struct NoArrayIndexKey;

impl NativeRule for NoArrayIndexKey {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-array-index-key".to_owned(),
            description: "Disallow usage of array index as key".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if this is a .map() call
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "map" {
            return;
        }

        // Get the callback argument
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        // Extract the index parameter name and body span from the callback
        let (param_name, body_start, body_end) = match first_arg {
            Argument::ArrowFunctionExpression(arrow) => {
                let Some(param) = arrow.params.items.get(1) else {
                    return;
                };
                let Some(name) = extract_param_name(param) else {
                    return;
                };
                (name, arrow.span.start, arrow.span.end)
            }
            Argument::FunctionExpression(func) => {
                let Some(param) = func.params.items.get(1) else {
                    return;
                };
                let Some(name) = extract_param_name(param) else {
                    return;
                };
                (name, func.span.start, func.span.end)
            }
            _ => return,
        };

        // Scan the callback body source text for `key={indexParam}` pattern
        let source = ctx.source_text();
        let start = usize::try_from(body_start).unwrap_or(0);
        let end = usize::try_from(body_end).unwrap_or(0);
        if start < source.len() && end <= source.len() && start < end {
            let body_source = &source[start..end];
            let key_pattern = format!("key={{{param_name}}}");
            if body_source.contains(&key_pattern) {
                ctx.report_warning(
                    "react/no-array-index-key",
                    &format!(
                        "Do not use array index `{param_name}` as `key` — use a stable identifier instead"
                    ),
                    Span::new(call.span.start, call.span.end),
                );
            }
        }
    }
}

/// Extract the binding identifier name from a formal parameter.
fn extract_param_name(param: &oxc_ast::ast::FormalParameter<'_>) -> Option<String> {
    match &param.pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str().to_owned()),
        _ => None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoArrayIndexKey)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_index_as_key_in_map() {
        let diags = lint(r"const x = items.map((item, index) => <div key={index}>{item}</div>);");
        assert_eq!(diags.len(), 1, "should flag array index used as key");
    }

    #[test]
    fn test_allows_stable_key() {
        let diags = lint(r"const x = items.map((item) => <div key={item.id}>{item.name}</div>);");
        assert!(diags.is_empty(), "stable key should not be flagged");
    }

    #[test]
    fn test_allows_map_without_key() {
        let diags = lint(r"const x = items.map((item) => <div>{item.name}</div>);");
        assert!(
            diags.is_empty(),
            "map without key should not be flagged by this rule"
        );
    }
}
