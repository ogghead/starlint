//! Rule: `jsdoc/match-name`
//!
//! Enforce `@name` tag matches the actual function/variable name.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

#[derive(Debug)]
pub struct MatchName;

/// Extract `@name` value from a `JSDoc` block.
fn extract_name_tag(block: &str) -> Option<String> {
    for line in block.lines() {
        let trimmed = super::trim_jsdoc_line(line);
        if let Some(rest) = trimmed.strip_prefix("@name") {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(
                    value
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_owned(),
                );
            }
        }
    }
    None
}

/// Extract the declared name from the code following a `JSDoc` block.
fn extract_declared_name(source: &str, after_pos: usize) -> Option<String> {
    let remaining = source.get(after_pos..).unwrap_or_default().trim_start();

    // function name(...)
    if let Some(fn_rest) = remaining.strip_prefix("function") {
        let fn_name_part = fn_rest.trim_start().trim_start_matches('*').trim_start();
        return fn_name_part
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .next()
            .filter(|s| !s.is_empty())
            .map(String::from);
    }

    // const/let/var name = ...
    for keyword in &["const ", "let ", "var "] {
        if let Some(rest) = remaining.strip_prefix(keyword) {
            return rest
                .trim_start()
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                .next()
                .filter(|s| !s.is_empty())
                .map(String::from);
        }
    }

    // class Name
    if let Some(rest) = remaining.strip_prefix("class ") {
        return rest
            .trim_start()
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .next()
            .filter(|s| !s.is_empty())
            .map(String::from);
    }

    None
}

impl NativeRule for MatchName {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jsdoc/match-name".to_owned(),
            description: "Enforce `@name` tag matches the declared name".to_owned(),
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

        let mut pos = 0;
        while let Some(start) = source.get(pos..).and_then(|s| s.find("/**")) {
            let abs_start = pos.saturating_add(start);
            let search_from = abs_start.saturating_add(3);
            if let Some(end) = source.get(search_from..).and_then(|s| s.find("*/")) {
                let abs_end = search_from.saturating_add(end).saturating_add(2);
                let block = source.get(abs_start..abs_end).unwrap_or_default();

                if let Some(doc_name) = extract_name_tag(block) {
                    if let Some(declared) = extract_declared_name(&source, abs_end) {
                        if doc_name != declared {
                            let span_start = u32::try_from(abs_start).unwrap_or(0);
                            let span_end = u32::try_from(abs_end).unwrap_or(span_start);
                            ctx.report(Diagnostic {
                                rule_name: "jsdoc/match-name".to_owned(),
                                message: format!(
                                    "`@name {doc_name}` does not match declared name `{declared}`"
                                ),
                                span: Span::new(span_start, span_end),
                                severity: Severity::Warning,
                                help: None,
                                fix: None,
                                labels: vec![],
                            });
                        }
                    }
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MatchName)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_mismatched_name() {
        let source = "/** @name wrong */\nfunction foo() {}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_matching_name() {
        let source = "/** @name foo */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_no_name_tag() {
        let source = "/** Some description */\nfunction foo() {}";
        let diags = lint(source);
        assert!(diags.is_empty());
    }
}
