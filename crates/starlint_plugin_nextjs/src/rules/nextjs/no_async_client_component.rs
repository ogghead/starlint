//! Rule: `nextjs/no-async-client-component`
//!
//! Forbid async function exports in client components (files with
//! `"use client"` directive). Client components cannot be async in React.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-async-client-component";

/// Flags async function exports in files with `"use client"` directive.
#[derive(Debug)]
pub struct NoAsyncClientComponent;

impl LintRule for NoAsyncClientComponent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid async client components".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        // Check for "use client" directive at the top of the file
        let has_use_client = source.lines().take(5).any(|line| {
            let trimmed = line.trim();
            trimmed == r#""use client""#
                || trimmed == r#""use client";"#
                || trimmed == "'use client'"
                || trimmed == "'use client';"
        });

        if !has_use_client {
            return;
        }

        // Scan for async function exports, computing fix positions up front
        #[allow(clippy::type_complexity)]
        let findings: Vec<(u32, u32, Option<(u32, u32)>)> = source
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim();
                let is_async_export = trimmed.starts_with("export async function ")
                    || trimmed.starts_with("export default async function ")
                    || trimmed.starts_with("export async function*");

                is_async_export.then(|| {
                    let line_offset: usize = source
                        .lines()
                        .take(idx)
                        .map(|l| l.len().saturating_add(1))
                        .sum();
                    let start = u32::try_from(line_offset).unwrap_or(0);
                    let end = u32::try_from(line_offset.saturating_add(trimmed.len())).unwrap_or(0);
                    let async_removal = trimmed.find("async ").map(|offset| {
                        let async_start = start.saturating_add(u32::try_from(offset).unwrap_or(0));
                        let async_end = async_start.saturating_add(6);
                        (async_start, async_end)
                    });
                    (start, end, async_removal)
                })
            })
            .collect();

        for (start, end, async_removal) in findings {
            let fix = async_removal.map(|(a_start, a_end)| Fix {
                kind: FixKind::SuggestionFix,
                message: "Remove `async` keyword".to_owned(),
                edits: vec![Edit {
                    span: Span::new(a_start, a_end),
                    replacement: String::new(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Client components cannot be async -- remove the `async` keyword or move to a server component".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Error,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoAsyncClientComponent)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_async_export_in_client_component() {
        let source = "\"use client\";\nexport async function Page() { return <div />; }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "async export in client component should be flagged"
        );
    }

    #[test]
    fn test_allows_sync_client_component() {
        let source = "\"use client\";\nexport function Page() { return <div />; }";
        let diags = lint(source);
        assert!(diags.is_empty(), "sync client component should pass");
    }

    #[test]
    fn test_allows_async_server_component() {
        let source = "export async function Page() { return <div />; }";
        let diags = lint(source);
        assert!(diags.is_empty(), "async server component should pass");
    }
}
