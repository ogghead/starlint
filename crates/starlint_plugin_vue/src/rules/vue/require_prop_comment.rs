//! Rule: `vue/require-prop-comment`
//!
//! Require `JSDoc` comments for props. Props are the public API of a component
//! and should be documented for maintainability.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/require-prop-comment";

/// Require `JSDoc` comments for props.
#[derive(Debug)]
pub struct RequirePropComment;

impl LintRule for RequirePropComment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require JSDoc comments for props".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text().to_owned();

        // Find props: { ... } block
        let Some(props_pos) = source.find("props:") else {
            return;
        };

        let after_props = source
            .get(props_pos.saturating_add(6)..)
            .unwrap_or_default()
            .trim_start();

        // Must start with `{` for object-style props
        if !after_props.starts_with('{') {
            return;
        }

        // Find matching closing brace
        let mut depth = 0_i32;
        let mut block_end = after_props.len();
        for (i, ch) in after_props.char_indices() {
            match ch {
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        block_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let block = after_props.get(1..block_end).unwrap_or_default();

        // Find property names at depth 0 within the block
        let mut i = 0;
        let block_bytes = block.as_bytes();
        let mut inner_depth = 0_i32;

        while i < block.len() {
            let ch = block_bytes.get(i).copied().unwrap_or(b' ');
            match ch {
                b'{' => inner_depth = inner_depth.saturating_add(1),
                b'}' => inner_depth = inner_depth.saturating_sub(1),
                _ if inner_depth == 0 && ch.is_ascii_alphabetic() => {
                    let start_i = i;
                    while i < block.len() {
                        let c = block_bytes.get(i).copied().unwrap_or(b' ');
                        if c.is_ascii_alphanumeric() || c == b'_' {
                            i = i.saturating_add(1);
                        } else {
                            break;
                        }
                    }
                    let prop_name = block.get(start_i..i).unwrap_or_default();

                    // Check if followed by `:` (object prop definition)
                    let after = block.get(i..).unwrap_or_default().trim_start();
                    if after.starts_with(':') || after.starts_with('{') {
                        // Check if preceded by a JSDoc comment
                        let before = block.get(..start_i).unwrap_or_default();
                        let trimmed = before.trim_end();
                        let has_jsdoc = trimmed.ends_with("*/") && trimmed.contains("/**");

                        if !has_jsdoc {
                            // Calculate absolute position
                            let abs_offset = source.get(..props_pos).unwrap_or_default().len();
                            let props_block_start = source
                                .get(props_pos.saturating_add(6)..)
                                .unwrap_or_default()
                                .len()
                                .saturating_sub(after_props.len());
                            let abs_pos = abs_offset
                                .saturating_add(6)
                                .saturating_add(props_block_start)
                                .saturating_add(1) // opening brace
                                .saturating_add(start_i);
                            let start = u32::try_from(abs_pos).unwrap_or(0);
                            let end =
                                start.saturating_add(u32::try_from(prop_name.len()).unwrap_or(0));
                            ctx.report(Diagnostic {
                                rule_name: RULE_NAME.to_owned(),
                                message: format!("Prop `{prop_name}` is missing a JSDoc comment"),
                                span: Span::new(start, end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
                    continue;
                }
                _ => {}
            }
            i = i.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(RequirePropComment);

    #[test]
    fn test_flags_undocumented_prop() {
        let source = r"export default { props: { title: { type: String } } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "undocumented prop should be flagged");
    }

    #[test]
    fn test_allows_documented_prop() {
        let source = r"export default { props: { /** The title */ title: { type: String } } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "documented prop should be allowed");
    }

    #[test]
    fn test_no_props_block() {
        let source = r"export default { data() { return {}; } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "no props block should produce no diags");
    }
}
