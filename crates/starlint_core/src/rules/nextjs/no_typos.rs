//! Rule: `nextjs/no-typos`
//!
//! Detect common Next.js API name typos. For example `getStaticPorps` instead
//! of `getStaticProps`. Uses text-based scanning for fast detection.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "nextjs/no-typos";

/// Known correct Next.js API names and their common typos.
const TYPO_PAIRS: &[(&str, &[&str])] = &[
    (
        "getStaticProps",
        &[
            "getStaticPorps",
            "getStaticprops",
            "getstaticProps",
            "getstaticprops",
            "getStatcProps",
            "getStaticPrps",
        ],
    ),
    (
        "getStaticPaths",
        &[
            "getStaticPahts",
            "getStaticpaths",
            "getstaticPaths",
            "getstaticpaths",
            "getStaticPath",
            "getStatcPaths",
        ],
    ),
    (
        "getServerSideProps",
        &[
            "getServerSidePorps",
            "getServerSideprops",
            "getserverSideProps",
            "getserversideprops",
            "getServerSidePrps",
            "getServersdieProps",
        ],
    ),
];

/// Flags common typos of Next.js API names.
#[derive(Debug)]
pub struct NoTypos;

impl NativeRule for NoTypos {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Detect common Next.js API typos".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        // Collect violations while source is borrowed, then report afterwards.
        let violations = {
            let source = ctx.source_text();

            // Early exit: skip files that don't contain any relevant prefix.
            // All typos start with one of these case variations.
            if !source.contains("getStatic")
                && !source.contains("getstatic")
                && !source.contains("getStatc")
                && !source.contains("getServer")
                && !source.contains("getserver")
            {
                return;
            }

            let mut hits: Vec<(&str, &str, Span)> = Vec::new();

            for (correct, typos) in TYPO_PAIRS {
                for typo in *typos {
                    let mut search_start = 0;
                    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(typo)) {
                        let abs_pos = search_start.saturating_add(pos);
                        let start = u32::try_from(abs_pos).unwrap_or(0);
                        let end = u32::try_from(abs_pos.saturating_add(typo.len())).unwrap_or(0);
                        hits.push((typo, correct, Span::new(start, end)));
                        search_start = abs_pos.saturating_add(typo.len());
                    }
                }
            }

            hits
        };

        for (typo, correct, span) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`{typo}` is a typo -- did you mean `{correct}`?"),
                span,
                severity: Severity::Error,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Rename to `{correct}`"),
                    edits: vec![Edit {
                        span,
                        replacement: (*correct).to_owned(),
                    }],
                    is_snippet: false,
                }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTypos)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_static_props_typo() {
        let diags = lint("export async function getStaticPorps() { return { props: {} }; }");
        assert_eq!(diags.len(), 1, "getStaticPorps typo should be flagged");
    }

    #[test]
    fn test_flags_server_side_props_typo() {
        let diags = lint("export async function getServerSidePorps() { return { props: {} }; }");
        assert_eq!(diags.len(), 1, "getServerSidePorps typo should be flagged");
    }

    #[test]
    fn test_allows_correct_api_names() {
        let diags = lint("export async function getStaticProps() { return { props: {} }; }");
        assert!(diags.is_empty(), "correct API names should not be flagged");
    }
}
