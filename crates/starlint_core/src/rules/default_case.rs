//! Rule: `default-case`
//!
//! Require `default` case in `switch` statements. A switch without a
//! default case may silently ignore unexpected values.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `switch` statements that lack a `default` case.
#[derive(Debug)]
pub struct DefaultCase;

impl NativeRule for DefaultCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "default-case".to_owned(),
            description: "Require default case in switch statements".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::SwitchStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::SwitchStatement(switch) = kind else {
            return;
        };

        let has_default = switch.cases.iter().any(|c| c.test.is_none());

        if !has_default {
            // Check for a "no default" comment in the last case
            let source = ctx.source_text();
            let start = usize::try_from(switch.span.start).unwrap_or(0);
            let end = usize::try_from(switch.span.end)
                .unwrap_or(0)
                .min(source.len());
            let switch_text = source.get(start..end).unwrap_or("");

            // Allow skipping default if there's a "no default" comment
            if switch_text.contains("no default") {
                return;
            }

            // Insert `default: break;` before the closing `}`
            let insert_pos = switch.span.end.saturating_sub(1);
            let fix = Some(Fix {
                kind: FixKind::SafeFix,
                message: "Add `default: break;`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(insert_pos, insert_pos),
                    replacement: " default: break; ".to_owned(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "default-case".to_owned(),
                message: "Expected a default case".to_owned(),
                span: Span::new(switch.span.start, switch.span.end),
                severity: Severity::Warning,
                help: None,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DefaultCase)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_missing_default() {
        let diags = lint("switch (x) { case 1: break; }");
        assert_eq!(diags.len(), 1, "missing default should be flagged");
    }

    #[test]
    fn test_allows_with_default() {
        let diags = lint("switch (x) { case 1: break; default: break; }");
        assert!(
            diags.is_empty(),
            "switch with default should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_default_comment() {
        let diags = lint("switch (x) { case 1: break; // no default\n}");
        assert!(
            diags.is_empty(),
            "switch with 'no default' comment should not be flagged"
        );
    }

    #[test]
    fn test_flags_empty_switch() {
        let diags = lint("switch (x) {}");
        assert_eq!(diags.len(), 1, "empty switch should be flagged");
    }
}
