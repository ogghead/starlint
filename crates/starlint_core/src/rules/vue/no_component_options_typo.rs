//! Rule: `vue/no-component-options-typo`
//!
//! Detect common typos in Vue component option names (e.g., `compued` instead
//! of `computed`, `destory` instead of `destroy`).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for NoComponentOptionsTypo {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Detect typos in Vue component option names".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

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
                    ctx.report(Diagnostic {
                        rule_name: RULE_NAME.to_owned(),
                        message: format!("Possible typo `{typo}` — did you mean `{correction}`?"),
                        span: Span::new(start, end),
                        severity: Severity::Warning,
                        help: None,
                        fix: Some(Fix {
                            message: format!("Rename to `{correction}`"),
                            edits: vec![Edit {
                                span: Span::new(start, end),
                                replacement: (*correction).to_owned(),
                            }],
                        }),
                        labels: vec![],
                    });
                }

                search_pos = abs_pos.saturating_add(typo.len());
            }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoComponentOptionsTypo)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

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
