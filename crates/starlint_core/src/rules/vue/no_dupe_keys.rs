//! Rule: `vue/no-dupe-keys`
//!
//! Forbid duplicate keys across `data`, `computed`, `methods`, and `props`
//! within a Vue component options object.

use std::collections::HashSet;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "vue/no-dupe-keys";

/// Sections to check for duplicate property names.
const SECTIONS: &[&str] = &["data", "computed", "methods", "props"];

/// Forbid duplicate keys in component options.
#[derive(Debug)]
pub struct NoDupeKeys;

/// Extract property names from a block following a section keyword.
fn extract_keys(source: &str, section_start: usize) -> Vec<(String, usize)> {
    let mut keys = Vec::new();
    let remaining = source.get(section_start..).unwrap_or_default();

    // Find the opening brace
    let Some(brace_pos) = remaining.find('{') else {
        return keys;
    };

    let block = remaining
        .get(brace_pos.saturating_add(1)..)
        .unwrap_or_default();
    let mut depth = 1_i32;
    let mut i = 0;

    while i < block.len() && depth > 0 {
        let ch = block.as_bytes().get(i).copied().unwrap_or(b' ');
        match ch {
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            _ if depth == 1 && ch.is_ascii_alphabetic() => {
                // Potential property name at depth 1
                let start_i = i;
                while i < block.len() {
                    let c = block.as_bytes().get(i).copied().unwrap_or(b' ');
                    if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                        i = i.saturating_add(1);
                    } else {
                        break;
                    }
                }
                let name = block.get(start_i..i).unwrap_or_default();
                // Check if followed by `:` or `(`
                let after = block.get(i..).unwrap_or_default().trim_start();
                if after.starts_with(':') || after.starts_with('(') {
                    let abs = section_start
                        .saturating_add(brace_pos)
                        .saturating_add(1)
                        .saturating_add(start_i);
                    keys.push((name.to_owned(), abs));
                }
                continue;
            }
            _ => {}
        }
        i = i.saturating_add(1);
    }

    keys
}

impl NativeRule for NoDupeKeys {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Forbid duplicate keys in component options".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text().to_owned();

        let mut all_keys: HashSet<String> = HashSet::new();

        for section in SECTIONS {
            let pattern = format!("{section}:");
            if let Some(pos) = source.find(&pattern) {
                let keys = extract_keys(&source, pos.saturating_add(pattern.len()));
                for (key, abs_pos) in keys {
                    if !all_keys.insert(key.clone()) {
                        let start = u32::try_from(abs_pos).unwrap_or(0);
                        let end = start.saturating_add(u32::try_from(key.len()).unwrap_or(0));
                        ctx.report_warning(
                            RULE_NAME,
                            &format!("Duplicate key `{key}` found across component options"),
                            Span::new(start, end),
                        );
                    }
                }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDupeKeys)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_duplicate_across_sections() {
        let source = r"export default { computed: { foo() {} }, methods: { foo() {} } };";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "duplicate key should be flagged");
    }

    #[test]
    fn test_allows_unique_keys() {
        let source = r"export default { computed: { foo() {} }, methods: { bar() {} } };";
        let diags = lint(source);
        assert!(diags.is_empty(), "unique keys should be allowed");
    }

    #[test]
    fn test_no_sections() {
        let source = r"export default { name: 'test' };";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "no relevant sections should produce no diags"
        );
    }
}
