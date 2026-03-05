//! Rule: `prefer-add-event-listener`
//!
//! Prefer `addEventListener` over assigning to `on*` event-handler
//! properties. Using `addEventListener` allows multiple handlers and
//! provides more control over event handling.

use oxc_ast::AstKind;
use oxc_ast::ast::AssignmentTarget;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `on*` event-handler property assignments.
#[derive(Debug)]
pub struct PreferAddEventListener;

impl NativeRule for PreferAddEventListener {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-add-event-listener".to_owned(),
            description: "Prefer `addEventListener` over `on*` property assignment".to_owned(),
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

        let AssignmentTarget::StaticMemberExpression(member) = &assign.left else {
            return;
        };

        let prop_name = member.property.name.as_str();

        if is_event_handler_property(prop_name) {
            // Extract event name: "onclick" -> "click"
            let event_name = prop_name.strip_prefix("on").unwrap_or(prop_name);

            // Build fix: el.onclick = handler -> el.addEventListener('click', handler)
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                let obj_span = member.object.span();
                let rhs_span = assign.right.span();
                let obj_text = source
                    .get(obj_span.start as usize..obj_span.end as usize)
                    .unwrap_or("");
                let rhs_text = source
                    .get(rhs_span.start as usize..rhs_span.end as usize)
                    .unwrap_or("");
                (!obj_text.is_empty() && !rhs_text.is_empty()).then(|| Fix {
                    message: format!(
                        "Replace with `{obj_text}.addEventListener('{event_name}', {rhs_text})`"
                    ),
                    edits: vec![Edit {
                        span: Span::new(assign.span.start, assign.span.end),
                        replacement: format!(
                            "{obj_text}.addEventListener('{event_name}', {rhs_text})"
                        ),
                    }],
                })
            };

            ctx.report(Diagnostic {
                rule_name: "prefer-add-event-listener".to_owned(),
                message: format!("Prefer `addEventListener` over assigning to `.{prop_name}`"),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Warning,
                help: Some(format!(
                    "Use `addEventListener('{event_name}', handler)` instead"
                )),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check if a property name matches the `on<event>` pattern.
///
/// The property must start with `on` followed by a lowercase ASCII letter
/// (e.g. `onclick`, `onload`, `onchange`). Properties like `onFoo` with
/// an uppercase letter after `on` are not considered standard DOM events.
fn is_event_handler_property(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("on") else {
        return false;
    };

    rest.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferAddEventListener)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_onclick_assignment() {
        let diags = lint("el.onclick = handler;");
        assert_eq!(diags.len(), 1, "el.onclick assignment should be flagged");
    }

    #[test]
    fn test_flags_window_onload() {
        let diags = lint("window.onload = init;");
        assert_eq!(diags.len(), 1, "window.onload assignment should be flagged");
    }

    #[test]
    fn test_flags_onchange() {
        let diags = lint("input.onchange = validate;");
        assert_eq!(
            diags.len(),
            1,
            "input.onchange assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_add_event_listener() {
        let diags = lint("el.addEventListener('click', handler);");
        assert!(diags.is_empty(), "addEventListener should not be flagged");
    }

    #[test]
    fn test_allows_uppercase_after_on() {
        let diags = lint("el.onFoo = bar;");
        assert!(
            diags.is_empty(),
            "onFoo with uppercase F is not a standard event handler"
        );
    }

    #[test]
    fn test_allows_non_on_property() {
        let diags = lint("el.value = 'test';");
        assert!(
            diags.is_empty(),
            "non-on property assignment should not be flagged"
        );
    }

    #[test]
    fn test_allows_identifier_assignment() {
        let diags = lint("onclick = handler;");
        assert!(
            diags.is_empty(),
            "bare identifier assignment should not be flagged"
        );
    }
}
