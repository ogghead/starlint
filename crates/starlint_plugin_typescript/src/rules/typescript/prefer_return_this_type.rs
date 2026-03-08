//! Rule: `typescript/prefer-return-this-type`
//!
//! Prefer returning `this` type over the class name in fluent / chainable
//! methods. When a class method returns `this`, the return type should be
//! declared as `this` rather than the class name — this allows subclasses
//! to inherit the correct return type without overriding.
//!
//! Simplified syntax-only version — full checking requires type information.
//!
//! This text-based heuristic scans for `class <Name>` declarations and then
//! checks for methods that have `: <Name>` as their explicit return type.
//!
//! Flagged patterns:
//! - `methodName(): ClassName { ... return this; }`
//!
//! Allowed patterns:
//! - `methodName(): this { ... return this; }`

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-return-this-type";

/// Flags class methods that return the class name instead of `this`.
#[derive(Debug)]
pub struct PreferReturnThisType;

impl LintRule for PreferReturnThisType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer `this` return type over the class name for chainable methods"
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
            find_class_name_return_types(source)
                .into_iter()
                .map(|(start, end, name_start, name_end)| Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message:
                        "Use `this` as the return type instead of the class name for chainable methods"
                            .to_owned(),
                    span: Span::new(start, end),
                    severity: Severity::Warning,
                    help: None,
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: "Replace class name with `this`".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(name_start, name_end),
                            replacement: "this".to_owned(),
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

/// Scan source text for class declarations and find methods that return the
/// class type by name instead of `this`.
///
/// Returns `(line_start, line_end, class_name_start, class_name_end)` for each
/// flagged line. The class name span points to just the class name token in the
/// return type annotation.
fn find_class_name_return_types(source: &str) -> Vec<(u32, u32, u32, u32)> {
    let mut results = Vec::new();

    // Phase 1: collect all class names
    let class_names = collect_class_names(source);
    if class_names.is_empty() {
        return results;
    }

    // Phase 2: scan for method signatures that return one of the class names
    let mut byte_offset: u32 = 0;

    for line in source.lines() {
        let line_len = u32::try_from(line.len()).unwrap_or(0);
        let trimmed = line.trim();

        // Check each class name for ): ClassName pattern
        for name in &class_names {
            let return_pattern = format!("): {name}");

            if trimmed.contains(&return_pattern) {
                // Ensure this looks like a method signature (has `(` before `)`)
                if let Some(paren_pos) = trimmed.find('(') {
                    if let Some(ret_pos) = trimmed.find(&return_pattern) {
                        if paren_pos < ret_pos {
                            let leading_ws =
                                u32::try_from(line.len().saturating_sub(trimmed.len()))
                                    .unwrap_or(0);
                            let start = byte_offset.saturating_add(leading_ws);
                            let end = byte_offset.saturating_add(line_len);
                            // Position of the class name within the line:
                            // ret_pos + 3 skips "): " to reach the class name
                            let name_offset = u32::try_from(ret_pos.saturating_add(3)).unwrap_or(0);
                            let name_len = u32::try_from(name.len()).unwrap_or(0);
                            let name_start = start.saturating_add(name_offset);
                            let name_end = name_start.saturating_add(name_len);
                            results.push((start, end, name_start, name_end));
                            break;
                        }
                    }
                }
            }
        }

        // +1 for the newline character
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    results
}

/// Extract class names from `class <Name>` declarations in source text.
fn collect_class_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let needle = "class ";

    let mut search_start: usize = 0;
    while let Some(pos) = source.get(search_start..).and_then(|s| s.find(needle)) {
        let abs_pos = search_start.saturating_add(pos);

        // Only match if `class` is at the start of a token (not inside another word)
        let is_word_start = abs_pos == 0
            || source
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_some_and(|&b| !b.is_ascii_alphanumeric() && b != b'_');

        if is_word_start {
            let name_start = abs_pos.saturating_add(needle.len());

            let name: String = source
                .get(name_start..)
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();

            if !name.is_empty() {
                names.push(name);
            }
        }

        search_start = abs_pos
            .saturating_add(needle.len())
            .max(search_start.saturating_add(1));
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferReturnThisType)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_method_returning_class_name() {
        let source =
            "class Builder {\n  setName(name: string): Builder {\n    return this;\n  }\n}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "method returning class name should be flagged"
        );
    }

    #[test]
    fn test_allows_method_returning_this() {
        let source = "class Builder {\n  setName(name: string): this {\n    return this;\n  }\n}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "method returning `this` type should not be flagged"
        );
    }

    #[test]
    fn test_allows_method_returning_different_type() {
        let source = "class Builder {\n  getName(): string {\n    return this.name;\n  }\n}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "method returning a different type should not be flagged"
        );
    }

    #[test]
    fn test_no_class_no_flags() {
        let source = "function build(): string { return ''; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "functions outside a class should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_classes() {
        let source = "class Foo {\n  chain(): Foo { return this; }\n}\nclass Bar {\n  chain(): Bar { return this; }\n}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            2,
            "both classes with self-returning methods should be flagged"
        );
    }
}
