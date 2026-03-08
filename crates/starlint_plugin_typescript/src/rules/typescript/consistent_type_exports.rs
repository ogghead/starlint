//! Rule: `typescript/consistent-type-exports`
//!
//! Require `export type` for type-only exports. When a name is declared with
//! `interface` or `type`, it should be exported using `export type { Name }`
//! rather than `export { Name }` to clearly communicate that only type
//! information is being exported.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for `interface` and `type` declarations,
//! then checks whether named exports of those identifiers use `export type`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/consistent-type-exports";

/// Flags `export { TypeName }` when `TypeName` was declared with `interface`
/// or `type`, suggesting `export type { TypeName }` instead.
#[derive(Debug)]
pub struct ConsistentTypeExports;

impl LintRule for ConsistentTypeExports {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Require `export type` for type-only exports".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let violations = find_type_only_exports(source);

        // Collect fix data into owned values to satisfy borrow checker
        let fixes: Vec<_> = violations
            .into_iter()
            .map(|(name, span)| {
                let line_text = source
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or("");
                let replacement = line_text.replacen("export {", "export type {", 1);
                (name, span, replacement)
            })
            .collect();

        for (name, span, replacement) in fixes {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Use `export type {{ {name} }}` instead of `export {{ {name} }}` — `{name}` is a type"),
                span,
                severity: Severity::Warning,
                help: Some("Add `type` keyword to export".to_owned()),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Add `type` keyword to export".to_owned(),
                    edits: vec![Edit {
                        span,
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Collect names declared with `interface` or `type` alias syntax.
///
/// Looks for patterns like `interface Foo` and `type Foo =`.
fn collect_type_declarations(source: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Match `interface Foo` or `export interface Foo`
        let interface_prefix = if trimmed.starts_with("interface ") {
            Some("interface ")
        } else if trimmed.starts_with("export interface ") {
            Some("export interface ")
        } else {
            None
        };

        if let Some(prefix) = interface_prefix {
            let rest = trimmed.get(prefix.len()..).unwrap_or("");
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();
            if !name.is_empty() {
                names.push(name);
            }
        }

        // Match `type Foo =` or `export type Foo =` (but not `export type {`)
        let type_prefix = if trimmed.starts_with("type ") {
            Some("type ")
        } else if trimmed.starts_with("export type ") && !trimmed.contains('{') {
            Some("export type ")
        } else {
            None
        };

        if let Some(prefix) = type_prefix {
            let rest = trimmed.get(prefix.len()..).unwrap_or("");
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();
            // Ensure it's a type alias (followed by `=`) and not just any `type` usage
            let after_name = rest.get(name.len()..).unwrap_or("").trim_start();
            if !name.is_empty() && after_name.starts_with('=') {
                names.push(name);
            }
        }
    }

    names
}

/// Find `export { Name }` statements where `Name` is a type-only declaration.
///
/// Returns a list of `(name, span)` pairs for each violation.
fn find_type_only_exports(source: &str) -> Vec<(String, Span)> {
    let type_names = collect_type_declarations(source);
    if type_names.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut byte_offset: usize = 0;

    for line in source.lines() {
        let trimmed = line.trim();
        let line_len = line.len();

        // Look for `export {` but NOT `export type {`
        if trimmed.starts_with("export {") && !trimmed.starts_with("export type ") {
            // Extract the specifiers between braces
            if let Some(brace_start) = trimmed.find('{') {
                if let Some(brace_end) = trimmed.find('}') {
                    let specifiers_str = trimmed
                        .get(brace_start.saturating_add(1)..brace_end)
                        .unwrap_or("");

                    let specifiers: Vec<&str> = specifiers_str
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .collect();

                    // Check if ALL exported names are type declarations
                    let all_are_types = !specifiers.is_empty()
                        && specifiers.iter().all(|spec| {
                            // Handle `Name as Alias` syntax — use the original name
                            let original_name = spec.split_whitespace().next().unwrap_or("");
                            type_names.iter().any(|t| t == original_name)
                        });

                    if all_are_types {
                        let start = u32::try_from(byte_offset).unwrap_or(0);
                        let end =
                            u32::try_from(byte_offset.saturating_add(line_len)).unwrap_or(start);
                        let exported_names = specifiers.join(", ");
                        results.push((exported_names, Span::new(start, end)));
                    }
                }
            }
        }

        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    use starlint_rule_framework::lint_source;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentTypeExports)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_interface_export_without_type() {
        let source = "interface Foo { x: number; }\nexport { Foo };";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "exporting an interface without `export type` should be flagged"
        );
    }

    #[test]
    fn test_flags_type_alias_export_without_type() {
        let source = "type Bar = string | number;\nexport { Bar };";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "exporting a type alias without `export type` should be flagged"
        );
    }

    #[test]
    fn test_allows_export_type_syntax() {
        let source = "interface Foo { x: number; }\nexport type { Foo };";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "`export type` syntax should not be flagged"
        );
    }

    #[test]
    fn test_allows_value_export() {
        let source = "const foo = 42;\nexport { foo };";
        let diags = lint(source);
        assert!(diags.is_empty(), "exporting a value should not be flagged");
    }

    #[test]
    fn test_allows_mixed_export() {
        let source = "interface Foo { x: number; }\nconst bar = 1;\nexport { Foo, bar };";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "mixed type and value exports should not be flagged"
        );
    }
}
