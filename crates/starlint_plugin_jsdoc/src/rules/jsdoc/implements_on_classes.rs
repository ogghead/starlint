//! Rule: `jsdoc/implements-on-classes`
//!
//! Enforce `@implements` is only used on class declarations.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

#[derive(Debug)]
pub struct ImplementsOnClasses;

/// Check if the code following a `JSDoc` block starts with a class declaration.
fn followed_by_class(source: &str, after_pos: usize) -> bool {
    let remaining = source.get(after_pos..).unwrap_or_default().trim_start();
    remaining.starts_with("class ")
        || remaining.starts_with("class{")
        || remaining.starts_with("abstract class ")
        || remaining.starts_with("export class ")
        || remaining.starts_with("export default class ")
        || remaining.starts_with("export abstract class ")
}

impl LintRule for ImplementsOnClasses {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/implements-on-classes".to_owned(),
            description: "Enforce `@implements` is only used on class declarations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        let mut pos = 0;
        while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
            let abs_start = pos.saturating_add(start);
            let search_from = abs_start.saturating_add(3);
            if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                let abs_end = search_from.saturating_add(end).saturating_add(2);
                let block = source.get(abs_start..abs_end).unwrap_or_default();

                let has_implements = block.lines().any(|line| {
                    let trimmed = super::trim_jsdoc_line(line);
                    trimmed.starts_with("@implements")
                });

                if has_implements && !followed_by_class(&source, abs_end) {
                    let span_start = u32::try_from(abs_start).unwrap_or(0);
                    let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                    ctx.report(Diagnostic {
                        rule_name: "jsdoc/implements-on-classes".to_owned(),
                        message: "`@implements` should only be used on class declarations"
                            .to_owned(),
                        span: Span::new(span_start, span_end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }

                pos = abs_end;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(ImplementsOnClasses);

    #[test]
    fn test_flags_implements_on_function() {
        let source = "/** @implements {Foo} */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_implements_on_class() {
        let source = "/** @implements {Foo} */\nclass Bar {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_implements_on_export_class() {
        let source = "/** @implements {Foo} */\nexport class Bar {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
