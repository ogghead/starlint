//! Rule: `typescript/prefer-readonly`
//!
//! Prefer `readonly` modifier for class properties that are never reassigned.
//! Properties that are only assigned in the constructor (or at declaration)
//! should be marked as `readonly` to signal immutability.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! This text-based heuristic scans class bodies for property declarations
//! that lack the `readonly` keyword and are not preceded by `static`.
//! It flags private/protected properties (using `private` or `#` prefix)
//! that could be `readonly`.
//!
//! Flagged patterns:
//! - `private name: string;`
//! - `private name = "value";`
//! - `#name: string;`
//!
//! Allowed patterns:
//! - `private readonly name: string;`
//! - `readonly #name: string;`
//! - `static name: string;`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-readonly";

/// Flags private/protected class properties that lack the `readonly` modifier.
#[derive(Debug)]
pub struct PreferReadonly;

impl LintRule for PreferReadonly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Prefer `readonly` modifier for class properties that are never reassigned"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let diagnostics = {
            let source = ctx.source_text();
            find_non_readonly_properties(source)
                .into_iter()
                .map(|(start, end, insert_pos)| Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message:
                        "This property could be `readonly` — consider adding the `readonly` modifier"
                            .to_owned(),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: None,
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Add `readonly` modifier".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(insert_pos, insert_pos),
                            replacement: "readonly ".to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                })
                .collect::<Vec<_>>()
        };

        for diag in diagnostics {
            ctx.report(diag);
        }
    }
}

/// Keywords that indicate a property declaration in a class body.
const VISIBILITY_KEYWORDS: &[&str] = &["private ", "protected "];

/// Scan source text for class property declarations that lack `readonly`.
///
/// Looks for lines inside class bodies that start with `private` or `protected`
/// (or use `#` private field syntax) without the `readonly` keyword, and that
/// contain either `:` (type annotation) or `=` (initializer).
///
/// Returns `(start, end, insert_pos)` for each occurrence, where `insert_pos`
/// is the absolute offset where `readonly ` should be inserted.
fn find_non_readonly_properties(source: &str) -> Vec<(u32, u32, u32)> {
    let mut results = Vec::new();
    let mut byte_offset: u32 = 0;

    for line in source.lines() {
        let line_len = u32::try_from(line.len()).unwrap_or(0);
        let trimmed = line.trim();

        // Skip lines that already have `readonly`
        if !trimmed.contains("readonly") {
            if let Some(insert_offset) = non_readonly_property_insert_offset(trimmed) {
                let leading_ws =
                    u32::try_from(line.len().saturating_sub(trimmed.len())).unwrap_or(0);
                let start = byte_offset.saturating_add(leading_ws);
                let end = byte_offset.saturating_add(line_len);
                let insert_pos = start.saturating_add(u32::try_from(insert_offset).unwrap_or(0));
                results.push((start, end, insert_pos));
            }
        }

        // +1 for the newline character
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    results
}

/// Check if a trimmed line looks like a class property declaration without
/// `readonly`. Returns the offset within the trimmed line where `readonly `
/// should be inserted, or `None` if the line is not a fixable property.
///
/// For `private name: string;` → insert after `private ` (offset 8)
/// For `protected count = 0;` → insert after `protected ` (offset 10)
/// For `#secret: string;` → insert at offset 0 (before `#`)
fn non_readonly_property_insert_offset(trimmed: &str) -> Option<usize> {
    // Skip static properties — they have different semantics
    if trimmed.starts_with("static ") {
        return None;
    }

    // Skip constructor lines
    if trimmed.starts_with("constructor") {
        return None;
    }

    // Check for visibility-keyword-prefixed properties: `private x: T;`
    for keyword in VISIBILITY_KEYWORDS {
        if trimmed.starts_with(keyword) {
            let after_keyword = trimmed.get(keyword.len()..).unwrap_or("");
            if is_property_body(after_keyword) {
                return Some(keyword.len());
            }
            return None;
        }
    }

    // Check for `#` private field syntax: `#name: T;` or `#name = value;`
    if trimmed.starts_with('#') {
        let after_hash = trimmed.get(1..).unwrap_or("");
        // Must start with a letter (field name)
        if after_hash
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
            && is_property_body(after_hash)
        {
            return Some(0);
        }
    }

    None
}

/// Check if text after the visibility keyword (or `#`) looks like a property
/// declaration (has `:` or `=`) and not a method (no `(`  before `:` or `=`).
fn is_property_body(text: &str) -> bool {
    // Skip `static` if it follows the visibility keyword
    if text.starts_with("static ") {
        return false;
    }

    // Find positions of key characters
    let paren_pos = text.find('(');
    let colon_pos = text.find(':');
    let eq_pos = text.find('=');

    // Must have a colon (type annotation) or `=` (initializer)
    let has_declaration = colon_pos.is_some() || eq_pos.is_some();
    if !has_declaration {
        return false;
    }

    // If there is a `(` before `:` or `=`, it is a method, not a property
    if let Some(pp) = paren_pos {
        let declaration_pos = colon_pos
            .unwrap_or(usize::MAX)
            .min(eq_pos.unwrap_or(usize::MAX));
        if pp < declaration_pos {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferReadonly)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_private_property_without_readonly() {
        let source = "class Foo {\n  private name: string;\n}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "private property without readonly should be flagged"
        );
    }

    #[test]
    fn test_flags_hash_private_field() {
        let source = "class Foo {\n  #secret: string;\n}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "private field with # syntax should be flagged"
        );
    }

    #[test]
    fn test_allows_private_readonly_property() {
        let source = "class Foo {\n  private readonly name: string;\n}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "private readonly property should not be flagged"
        );
    }

    #[test]
    fn test_allows_method_declaration() {
        let source = "class Foo {\n  private getName(): string { return ''; }\n}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "method declarations should not be flagged"
        );
    }

    #[test]
    fn test_flags_protected_property() {
        let source = "class Foo {\n  protected count = 0;\n}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "protected property without readonly should be flagged"
        );
    }
}
