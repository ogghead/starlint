//! Rule: `vue/no-component-options-typo`
//!
//! Detect common typos in Vue component option names (e.g., `compued` instead
//! of `computed`, `destory` instead of `destroy`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-component-options-typo";

/// Common typos mapped to their corrections.
const TYPO_MAP: &[(&str, &str)] = &[
    ("compued", "computed"),
    ("comptued", "computed"),
    ("computde", "computed"),
    ("metods", "methods"),
    ("methds", "methods"),
    ("mehods", "methods"),
    ("crated", "created"),
    ("craeted", "created"),
    ("mountd", "mounted"),
    ("moutned", "mounted"),
    ("destory", "destroy"),
    ("destoryed", "destroyed"),
    ("beforeDestory", "beforeDestroy"),
    ("beforeMout", "beforeMount"),
    ("compoents", "components"),
    ("componets", "components"),
    ("diretives", "directives"),
    ("watcg", "watch"),
    ("wtach", "watch"),
    ("prosp", "props"),
    ("propss", "props"),
];

/// Detect typos in component options.
#[derive(Debug)]
pub struct NoComponentOptionsTypo;

impl LintRule for NoComponentOptionsTypo {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Detect typos in Vue component option names".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        // Pre-filter: only scan 21 typo patterns if the file looks like a Vue component.
        // This avoids 21 contains scans on files that can't have Vue component options.
        (source_text.contains("export default") || source_text.contains("defineComponent"))
            && TYPO_MAP.iter().any(|(typo, _)| source_text.contains(typo))
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations: Vec<(&str, &str, u32, u32)> = {
            let source = ctx.source_text();
            let mut found = Vec::new();
            for &(typo, correction) in TYPO_MAP {
                let mut search_pos = 0;
                while let Some(offset) = source.get(search_pos..).and_then(|s| s.find(typo)) {
                    let abs_pos = search_pos.saturating_add(offset);

                    // Verify it looks like a property key (followed by `:` or `(`)
                    let after = source
                        .get(abs_pos.saturating_add(typo.len())..)
                        .unwrap_or_default()
                        .trim_start();

                    if after.starts_with(':') || after.starts_with('(') {
                        let start = u32::try_from(abs_pos).unwrap_or(0);
                        let end = start.saturating_add(u32::try_from(typo.len()).unwrap_or(0));
                        found.push((typo, correction, start, end));
                    }

                    search_pos = abs_pos.saturating_add(typo.len());
                }
            }
            found
        };

        for (typo, correction, start, end) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Possible typo `{typo}` — did you mean `{correction}`?"),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Rename to `{correction}`"),
                    edits: vec![Edit {
                        span: Span::new(start, end),
                        replacement: (*correction).to_owned(),
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
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoComponentOptionsTypo);

    #[test]
    fn test_flags_compued_typo() {
        let source = r"export default { compued: { x() { return 1; } } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "compued typo should be flagged");
    }

    #[test]
    fn test_allows_correct_computed() {
        let source = r"export default { computed: { x() { return 1; } } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "correct option should not be flagged");
    }

    #[test]
    fn test_flags_metods_typo() {
        let source = r"export default { metods: { foo() {} } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "metods typo should be flagged");
    }
}
