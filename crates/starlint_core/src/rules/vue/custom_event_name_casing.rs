//! Rule: `vue/custom-event-name-casing`
//!
//! Enforce camelCase for custom event names in `$emit()` calls.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/custom-event-name-casing";

/// Enforce camelCase for custom event names in `$emit()`.
#[derive(Debug)]
pub struct CustomEventNameCasing;

/// Convert a string to `camelCase`.
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '-' || ch == '_' || ch == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Check if a string is `camelCase`.
fn is_camel_case(s: &str) -> bool {
    let first = s.chars().next();
    matches!(first, Some('a'..='z')) && !s.contains('-') && !s.contains('_') && !s.contains(' ')
}

impl NativeRule for CustomEventNameCasing {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce camelCase for custom event names in `$emit()`".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for this.$emit() or $emit()
        let is_emit = match &call.callee {
            Expression::StaticMemberExpression(member) => member.property.name.as_str() == "$emit",
            Expression::Identifier(ident) => ident.name.as_str() == "$emit",
            _ => false,
        };

        if !is_emit {
            return;
        }

        // Check first argument for casing — it should be a string literal
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let event_name = match first_arg {
            oxc_ast::ast::Argument::StringLiteral(lit) => lit.value.as_str(),
            _ => return,
        };

        if !event_name.is_empty() && !is_camel_case(event_name) {
            // Fix: convert event name to camelCase
            let camel = to_camel_case(event_name);
            let fix = (camel != event_name).then(|| {
                // Replace just the string content (inside the quotes)
                let arg_span = first_arg.span();
                // The string literal span includes quotes, so skip them
                let inner_start = arg_span.start.saturating_add(1);
                let inner_end = arg_span.end.saturating_sub(1);
                Fix {
                    message: format!("Rename to `{camel}`"),
                    edits: vec![Edit {
                        span: Span::new(inner_start, inner_end),
                        replacement: camel.clone(),
                    }],
                    is_snippet: false,
                }
            });

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Custom event name `{event_name}` should be camelCase"),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: Some(format!("Rename to `{camel}`")),
                fix,
                labels: vec![],
            });
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CustomEventNameCasing)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_camel_case_event() {
        let source = r#"this.$emit("myEvent", value);"#;
        let diags = lint(source);
        assert!(diags.is_empty(), "camelCase event name should be allowed");
    }

    #[test]
    fn test_flags_kebab_case_event() {
        let source = r#"this.$emit("my-event", value);"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "kebab-case event name should be flagged");
    }

    #[test]
    fn test_flags_pascal_case_event() {
        let source = r#"this.$emit("MyEvent", value);"#;
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "PascalCase event name should be flagged");
    }
}
