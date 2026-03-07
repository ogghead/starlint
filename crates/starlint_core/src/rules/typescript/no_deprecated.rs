//! Rule: `typescript/no-deprecated`
//!
//! Disallow use of deprecated APIs. Flags usage of identifiers that are
//! marked with a `@deprecated` `JSDoc` tag in the same file.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for `JSDoc` `@deprecated` markers, extracts
//! the associated declaration name, and then checks if that name is used
//! elsewhere in the file.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-deprecated";

/// Flags usage of identifiers marked with `@deprecated` `JSDoc` tags.
#[derive(Debug)]
pub struct NoDeprecated;

impl LintRule for NoDeprecated {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow use of deprecated APIs".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        let deprecated_names = collect_deprecated_names(source);
        if deprecated_names.is_empty() {
            return;
        }

        let violations = find_deprecated_usages(source, &deprecated_names);

        for (name, span) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`{name}` is deprecated"),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Information about a deprecated declaration.
struct DeprecatedDecl {
    /// The name of the deprecated identifier.
    name: String,
    /// Byte offset of the declaration line start (used to skip the declaration itself).
    decl_line_start: usize,
    /// Byte offset of the declaration line end.
    decl_line_end: usize,
}

/// Scan source text for `JSDoc` `@deprecated` tags and extract the name of the
/// declaration that follows the comment block.
///
/// Looks for `/** @deprecated */` or multi-line `JSDoc` blocks containing
/// `@deprecated`, then extracts the identifier from the next declaration line
/// (function, const, let, var, class, interface, type, enum).
fn collect_deprecated_names(source: &str) -> Vec<DeprecatedDecl> {
    let mut results = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut byte_offset: usize = 0;
    let mut offsets: Vec<usize> = Vec::with_capacity(lines.len());

    // Pre-compute byte offsets for each line
    for line in &lines {
        offsets.push(byte_offset);
        byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
    }

    let mut line_idx: usize = 0;
    while line_idx < lines.len() {
        let trimmed = lines.get(line_idx).copied().unwrap_or("").trim();

        // Check for single-line JSDoc: `/** @deprecated */`
        let is_single_line_deprecated = trimmed.starts_with("/**")
            && trimmed.contains("@deprecated")
            && trimmed.ends_with("*/");

        // Check for multi-line JSDoc block start
        let is_block_start = trimmed.starts_with("/**") && !trimmed.ends_with("*/");

        if is_single_line_deprecated {
            // The declaration should be on the next non-empty line
            let next_line_idx = line_idx.saturating_add(1);
            if let Some(decl) = extract_declaration_name(&lines, &offsets, next_line_idx) {
                results.push(decl);
            }
        } else if is_block_start {
            // Scan the JSDoc block for @deprecated
            let mut found_deprecated = false;
            let mut block_end_idx = line_idx;

            let mut scan_idx = line_idx;
            while scan_idx < lines.len() {
                let scan_line = lines.get(scan_idx).copied().unwrap_or("").trim();
                if scan_line.contains("@deprecated") {
                    found_deprecated = true;
                }
                if scan_line.contains("*/") {
                    block_end_idx = scan_idx;
                    break;
                }
                scan_idx = scan_idx.saturating_add(1);
            }

            if found_deprecated {
                let next_line_idx = block_end_idx.saturating_add(1);
                if let Some(decl) = extract_declaration_name(&lines, &offsets, next_line_idx) {
                    results.push(decl);
                }
            }

            line_idx = block_end_idx;
        }

        line_idx = line_idx.saturating_add(1);
    }

    results
}

/// Extract the declaration name from a line at the given index.
///
/// Recognizes `function`, `const`, `let`, `var`, `class`, `interface`,
/// `type`, and `enum` declarations, including those prefixed with `export`.
fn extract_declaration_name(
    lines: &[&str],
    offsets: &[usize],
    line_idx: usize,
) -> Option<DeprecatedDecl> {
    let line = lines.get(line_idx).copied()?;
    let trimmed = line.trim();
    let line_start = offsets.get(line_idx).copied().unwrap_or(0);
    let line_end = line_start.saturating_add(line.len());

    #[allow(clippy::items_after_statements)]
    const KEYWORDS: &[&str] = &[
        "function ",
        "const ",
        "let ",
        "var ",
        "class ",
        "interface ",
        "type ",
        "enum ",
        "async function ",
    ];

    // Strip optional `export` or `export default` prefix
    let without_export = if trimmed.starts_with("export default ") {
        trimmed.get("export default ".len()..).unwrap_or("")
    } else if trimmed.starts_with("export ") {
        trimmed.get("export ".len()..).unwrap_or("")
    } else {
        trimmed
    };

    for keyword in KEYWORDS {
        if without_export.starts_with(keyword) {
            let rest = without_export.get(keyword.len()..).unwrap_or("");
            // Handle `function*` for generators
            let name_source = if rest.starts_with('*') {
                rest.get(1..).unwrap_or("").trim_start()
            } else {
                rest
            };

            let name: String = name_source
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();

            if !name.is_empty() {
                return Some(DeprecatedDecl {
                    name,
                    decl_line_start: line_start,
                    decl_line_end: line_end,
                });
            }
        }
    }

    None
}

/// Find usages of deprecated names in the source, excluding the declaration line itself.
///
/// Returns `(name, span)` for each usage found.
fn find_deprecated_usages(source: &str, deprecated: &[DeprecatedDecl]) -> Vec<(String, Span)> {
    let mut results = Vec::new();

    for decl in deprecated {
        let name = &decl.name;
        let mut search_from: usize = 0;

        while let Some(pos) = source
            .get(search_from..)
            .and_then(|s| s.find(name.as_str()))
        {
            let abs_pos = search_from.saturating_add(pos);
            let after_name = abs_pos.saturating_add(name.len());

            // Ensure it's a word boundary (not part of a larger identifier)
            let before_ok = abs_pos == 0
                || source
                    .as_bytes()
                    .get(abs_pos.saturating_sub(1))
                    .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

            let after_ok = source
                .as_bytes()
                .get(after_name)
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_' && *b != b'$');

            // Skip if this occurrence is on the declaration line itself
            let on_decl_line = abs_pos >= decl.decl_line_start && abs_pos < decl.decl_line_end;

            // Skip if inside a comment (simple heuristic: check for `//` or `/*` before on same line)
            let in_comment = is_in_comment(source, abs_pos);

            if before_ok && after_ok && !on_decl_line && !in_comment {
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = u32::try_from(after_name).unwrap_or(start);
                results.push((name.clone(), Span::new(start, end)));
            }

            search_from = after_name;
        }
    }

    results
}

/// Simple heuristic to check if a position is inside a comment.
fn is_in_comment(source: &str, pos: usize) -> bool {
    let before = source.get(..pos).unwrap_or("");

    // Check line comment
    if let Some(last_newline) = before.rfind('\n') {
        let line_segment = before.get(last_newline..).unwrap_or("");
        if line_segment.contains("//") {
            return true;
        }
    } else if before.contains("//") {
        return true;
    }

    // Check block comment
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
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoDeprecated)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_deprecated_function_usage() {
        let source = r"
/** @deprecated */
function oldFunc() { return 1; }
const x = oldFunc();
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "usage of a deprecated function should be flagged"
        );
    }

    #[test]
    fn test_flags_deprecated_const_usage() {
        let source = r"
/** @deprecated Use newValue instead */
const oldValue = 42;
console.log(oldValue);
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "usage of a deprecated constant should be flagged"
        );
    }

    #[test]
    fn test_flags_multiline_jsdoc_deprecated() {
        let source = r"
/**
 * This function is old.
 * @deprecated Use newFunc instead.
 */
function legacyFunc() { return 1; }
const y = legacyFunc();
";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "usage of a function with multiline JSDoc @deprecated should be flagged"
        );
    }

    #[test]
    fn test_allows_non_deprecated_usage() {
        let source = r"
function safeFunc() { return 1; }
const x = safeFunc();
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "usage of a non-deprecated function should not be flagged"
        );
    }

    #[test]
    fn test_allows_deprecated_declaration_itself() {
        let source = r"
/** @deprecated */
function oldFunc() { return 1; }
";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "the declaration of a deprecated function itself should not be flagged"
        );
    }
}
