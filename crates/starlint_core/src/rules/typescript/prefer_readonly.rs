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

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-readonly";

/// Flags private/protected class properties that lack the `readonly` modifier.
#[derive(Debug)]
pub struct PreferReadonly;

impl NativeRule for PreferReadonly {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Prefer `readonly` modifier for class properties that are never reassigned"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_non_readonly_properties(source);

        for (start, end) in findings {
            ctx.report_warning(
                RULE_NAME,
                "This property could be `readonly` — consider adding the `readonly` modifier",
                Span::new(start, end),
            );
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
/// Returns a list of `(start_offset, end_offset)` for each occurrence.
fn find_non_readonly_properties(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut byte_offset: u32 = 0;

    for line in source.lines() {
        let line_len = u32::try_from(line.len()).unwrap_or(0);
        let trimmed = line.trim();

        // Skip lines that already have `readonly`
        if !trimmed.contains("readonly") {
            let is_property = is_non_readonly_property_line(trimmed);
            if is_property {
                let leading_ws =
                    u32::try_from(line.len().saturating_sub(trimmed.len())).unwrap_or(0);
                let start = byte_offset.saturating_add(leading_ws);
                let end = byte_offset.saturating_add(line_len);
                results.push((start, end));
            }
        }

        // +1 for the newline character
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    results
}

/// Check if a trimmed line looks like a class property declaration without
/// `readonly` — either using a visibility keyword or `#` private field syntax.
///
/// Returns `true` for lines like:
/// - `private name: string;`
/// - `protected count = 0;`
/// - `#secret: string;`
///
/// Returns `false` for lines like:
/// - `private readonly name: string;` (already readonly)
/// - `static count = 0;` (static, not instance property)
/// - `private getName() {` (method declaration)
/// - `constructor(` (constructor)
fn is_non_readonly_property_line(trimmed: &str) -> bool {
    // Skip static properties — they have different semantics
    if trimmed.starts_with("static ") {
        return false;
    }

    // Skip constructor lines
    if trimmed.starts_with("constructor") {
        return false;
    }

    // Check for visibility-keyword-prefixed properties: `private x: T;`
    for keyword in VISIBILITY_KEYWORDS {
        if trimmed.starts_with(keyword) {
            let after_keyword = trimmed.get(keyword.len()..).unwrap_or("");
            return is_property_body(after_keyword);
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
        {
            return is_property_body(after_hash);
        }
    }

    false
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferReadonly)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
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
