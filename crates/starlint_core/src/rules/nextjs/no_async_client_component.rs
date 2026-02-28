//! Rule: `nextjs/no-async-client-component`
//!
//! Forbid async function exports in client components (files with
//! `"use client"` directive). Client components cannot be async in React.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-async-client-component";

/// Flags async function exports in files with `"use client"` directive.
#[derive(Debug)]
pub struct NoAsyncClientComponent;

impl NativeRule for NoAsyncClientComponent {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid async client components".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
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

        // Scan for async function exports
        let findings: Vec<(u32, u32)> = source
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
                    (start, end)
                })
            })
            .collect();

        for (start, end) in findings {
            ctx.report_error(
                RULE_NAME,
                "Client components cannot be async -- remove the `async` keyword or move to a server component",
                Span::new(start, end),
            );
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAsyncClientComponent)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
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
