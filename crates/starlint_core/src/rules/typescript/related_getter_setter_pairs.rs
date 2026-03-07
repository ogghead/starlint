//! Rule: `typescript/related-getter-setter-pairs`
//!
//! Require matching getter and setter pairs in classes and object literals.
//! A getter without a corresponding setter (or vice versa) is often a mistake
//! that leads to read-only or write-only properties with no compile-time
//! warning.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags getter/setter declarations that lack a matching counterpart.
#[derive(Debug)]
pub struct RelatedGetterSetterPairs;

impl LintRule for RelatedGetterSetterPairs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/related-getter-setter-pairs".to_owned(),
            description: "Require matching getter and setter pairs in classes and object literals"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let findings = find_unpaired_accessors(ctx.source_text());

        for (kind, name, start, end) in findings {
            let missing = if kind == "get" { "setter" } else { "getter" };
            ctx.report(Diagnostic {
                rule_name: "typescript/related-getter-setter-pairs".to_owned(),
                message: format!("Property `{name}` has a {kind}ter but no matching {missing}"),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// A detected accessor declaration.
struct Accessor {
    /// `"get"` or `"set"`.
    kind: &'static str,
    /// The property name.
    name: String,
    /// Byte offset of the accessor keyword in source.
    start: u32,
    /// Byte offset of the end of the property name.
    end: u32,
}

/// Scan source text for `get <name>(` and `set <name>(` patterns and return
/// any that lack a matching counterpart.
///
/// Returns a list of `(kind, name, start_offset, end_offset)` tuples for
/// each unpaired accessor.
fn find_unpaired_accessors(source: &str) -> Vec<(&'static str, String, u32, u32)> {
    let accessors = collect_accessors(source);

    let mut results = Vec::new();
    for accessor in &accessors {
        let counterpart = if accessor.kind == "get" { "set" } else { "get" };
        let has_pair = accessors
            .iter()
            .any(|a| a.kind == counterpart && a.name == accessor.name);
        if !has_pair {
            results.push((
                accessor.kind,
                accessor.name.clone(),
                accessor.start,
                accessor.end,
            ));
        }
    }
    results
}

/// Collect all `get <name>(` and `set <name>(` patterns from source text.
fn collect_accessors(source: &str) -> Vec<Accessor> {
    let mut accessors = Vec::new();

    for keyword in &["get", "set"] {
        let kind: &'static str = if *keyword == "get" { "get" } else { "set" };
        let pattern = *keyword;
        let pattern_len = pattern.len();

        let mut search_from: usize = 0;
        while let Some(pos) = source.get(search_from..).and_then(|s| s.find(pattern)) {
            let absolute_pos = search_from.saturating_add(pos);
            let after_keyword = absolute_pos.saturating_add(pattern_len);

            // The character before the keyword must be a boundary (whitespace, newline,
            // start of file, or opening brace)
            let valid_before = if absolute_pos == 0 {
                true
            } else {
                source
                    .as_bytes()
                    .get(absolute_pos.saturating_sub(1))
                    .is_none_or(|&b| {
                        b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == b'{'
                    })
            };

            if !valid_before {
                search_from = after_keyword;
                continue;
            }

            // After the keyword there must be a space followed by an identifier and `(`
            let rest = source.get(after_keyword..).unwrap_or("");
            if let Some(name) = extract_accessor_name(rest) {
                let name_end = after_keyword.saturating_add(rest.find('(').unwrap_or(0));
                let start = u32::try_from(absolute_pos).unwrap_or(0);
                let end = u32::try_from(name_end).unwrap_or(start);
                accessors.push(Accessor {
                    kind,
                    name: name.to_owned(),
                    start,
                    end,
                });
                search_from = name_end;
            } else {
                search_from = after_keyword;
            }
        }
    }

    accessors
}

/// Extract the property name following an accessor keyword.
///
/// Expects the input to start right after `get` or `set`. Looks for
/// ` <identifier>(` and returns the identifier if found.
fn extract_accessor_name(rest: &str) -> Option<&str> {
    // Must start with at least one space
    let trimmed = rest.strip_prefix(' ')?;

    // Find the opening paren
    let paren_pos = trimmed.find('(')?;
    let name = trimmed.get(..paren_pos)?.trim();

    // Validate that name is a simple identifier (alphanumeric + underscore)
    (!name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$'))
    .then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(RelatedGetterSetterPairs)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_getter_without_setter() {
        let diags = lint("class Foo { get value() { return 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "getter without matching setter should be flagged"
        );
    }

    #[test]
    fn test_flags_setter_without_getter() {
        let diags = lint("class Bar { set value(v: number) { this._v = v; } }");
        assert_eq!(
            diags.len(),
            1,
            "setter without matching getter should be flagged"
        );
    }

    #[test]
    fn test_allows_matching_getter_setter_pair() {
        let diags =
            lint("class Baz { get value() { return 1; } set value(v: number) { this._v = v; } }");
        assert!(
            diags.is_empty(),
            "matching getter/setter pair should not be flagged"
        );
    }

    #[test]
    fn test_flags_object_literal_getter_only() {
        let diags = lint("const obj = { get name() { return ''; } };");
        assert_eq!(
            diags.len(),
            1,
            "object literal with getter only should be flagged"
        );
    }

    #[test]
    fn test_allows_object_literal_with_both() {
        let diags = lint("const obj = { get name() { return ''; }, set name(v: string) { } };");
        assert!(
            diags.is_empty(),
            "object literal with both getter and setter should not be flagged"
        );
    }
}
