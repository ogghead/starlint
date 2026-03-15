//! Rule: `typescript/no-unnecessary-qualifier`
//!
//! Disallow unnecessary namespace qualifiers. When code is already inside a
//! namespace block, references to members of that same namespace do not need
//! the namespace prefix. For example, inside `namespace Foo { ... }`, writing
//! `Foo.bar` is redundant — `bar` alone suffices and is clearer.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags unnecessary namespace qualifiers where the reference is already
/// inside the namespace being qualified.
#[derive(Debug)]
pub struct NoUnnecessaryQualifier;

impl LintRule for NoUnnecessaryQualifier {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-qualifier".to_owned(),
            description: "Disallow unnecessary namespace qualifiers".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("namespace ") || source_text.contains("module ")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_unnecessary_qualifiers(source);

        for (ns_name, qual_start, qual_end) in findings {
            let span = Span::new(qual_start, qual_end);
            let message = format!(
                "Unnecessary namespace qualifier `{ns_name}.` — already inside namespace `{ns_name}`"
            );
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unnecessary-qualifier".to_owned(),
                message: message.clone(),
                span,
                severity: Severity::Warning,
                help: Some(format!("Remove the `{ns_name}.` qualifier")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Remove `{ns_name}.` qualifier"),
                    edits: vec![Edit {
                        span,
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Find patterns where `namespace X { ... X.something ... }` occurs.
///
/// Returns `(namespace_name, start, end)` for each unnecessary qualifier found.
fn find_unnecessary_qualifiers(source: &str) -> Vec<(String, u32, u32)> {
    let mut results = Vec::new();

    // Find namespace blocks and look for qualified references within them.
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos: usize = 0;

    while pos < len {
        // Look for `namespace` or `module` keyword
        let remaining = source.get(pos..).unwrap_or("");
        let ns_keyword = if remaining.starts_with("namespace ") {
            Some("namespace")
        } else if remaining.starts_with("module ") {
            Some("module")
        } else {
            None
        };

        if let Some(keyword) = ns_keyword {
            let after_keyword = pos.saturating_add(keyword.len().saturating_add(1));
            if let Some(ns_name) = extract_identifier(source, after_keyword) {
                // Find the opening brace
                let search_start = after_keyword.saturating_add(ns_name.len());
                if let Some(brace_offset) = source.get(search_start..).and_then(|s| s.find('{')) {
                    let brace_pos = search_start.saturating_add(brace_offset);
                    if let Some(close_pos) = find_matching_brace(source, brace_pos) {
                        // Search within the namespace body for `NsName.` references
                        let body_start = brace_pos.saturating_add(1);
                        let body = source.get(body_start..close_pos).unwrap_or("");
                        let qualifier = format!("{ns_name}.");

                        let mut search_from: usize = 0;
                        while let Some(idx) =
                            body.get(search_from..).and_then(|s| s.find(&qualifier))
                        {
                            let abs_pos =
                                body_start.saturating_add(search_from).saturating_add(idx);

                            // Ensure the qualifier is not preceded by another identifier
                            // character (would mean it's part of a longer name).
                            let is_start_of_token = abs_pos == 0
                                || source
                                    .as_bytes()
                                    .get(abs_pos.saturating_sub(1))
                                    .is_none_or(|&b| !is_ident_char(b));

                            if is_start_of_token && !is_inside_string_or_comment(source, abs_pos) {
                                let start = u32::try_from(abs_pos).unwrap_or(0);
                                let end = u32::try_from(abs_pos.saturating_add(qualifier.len()))
                                    .unwrap_or(start);
                                results.push((ns_name.clone(), start, end));
                            }

                            search_from = search_from
                                .saturating_add(idx)
                                .saturating_add(qualifier.len());
                        }

                        pos = close_pos.saturating_add(1);
                        continue;
                    }
                }
            }
        }

        pos = pos.saturating_add(1);
    }

    results
}

/// Extract an identifier starting at the given position.
fn extract_identifier(source: &str, start: usize) -> Option<String> {
    let remaining = source.get(start..)?;
    let end = remaining
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(remaining.len());
    if end == 0 {
        return None;
    }
    remaining.get(..end).map(String::from)
}

/// Check if a byte is a valid identifier character.
const fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Find the matching closing brace for an opening brace at `open_pos`.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth: usize = 0;
    let mut pos = open_pos;

    while pos < bytes.len() {
        match bytes.get(pos) {
            Some(b'{') => depth = depth.saturating_add(1),
            Some(b'}') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos = pos.saturating_add(1);
    }

    None
}

/// Simple heuristic check whether a position is inside a string or comment.
fn is_inside_string_or_comment(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");

    // Check for line comment
    if let Some(last_newline) = before.rfind('\n') {
        let line_before = before.get(last_newline..).unwrap_or("");
        if line_before.contains("//") {
            return true;
        }
    } else if before.contains("//") {
        return true;
    }

    // Check for block comment
    if let Some(block_start) = before.rfind("/*") {
        let between = before.get(block_start..).unwrap_or("");
        if !between.contains("*/") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoUnnecessaryQualifier, "test.ts");

    #[test]
    fn test_flags_qualified_reference_inside_namespace() {
        let source = "namespace Foo { const x = Foo.bar; }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "qualified reference inside own namespace should be flagged"
        );
    }

    #[test]
    fn test_allows_unqualified_reference() {
        let source = "namespace Foo { const x = bar; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "unqualified reference should not be flagged"
        );
    }

    #[test]
    fn test_allows_qualified_reference_outside_namespace() {
        let source = "namespace Foo { const x = 1; }\nconst y = Foo.bar;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "qualified reference outside the namespace should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_qualified_references() {
        let source = "namespace NS { const a = NS.x; const b = NS.y; }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            2,
            "multiple qualified references inside own namespace should all be flagged"
        );
    }

    #[test]
    fn test_allows_different_namespace_qualifier() {
        let source = "namespace Foo { const x = Bar.baz; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "qualifier for a different namespace should not be flagged"
        );
    }
}
