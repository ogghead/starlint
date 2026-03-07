//! Rule: `typescript/no-unnecessary-type-arguments`
//!
//! Disallow type arguments that match the default type parameter. When a
//! generic type parameter has a default, passing the same type as the default
//! is redundant and adds noise.
//!
//! Simplified syntax-only version — full checking requires type information.
//! This text-based heuristic scans for generic type parameter defaults of the
//! form `<T = SomeType>` and then flags usage sites that pass `SomeType` as
//! the type argument (e.g. `Foo<SomeType>`) when it matches the default.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-unnecessary-type-arguments";

/// A generic type with a single default parameter.
struct GenericDefault {
    /// The name of the generic type (e.g. `MyMap`).
    type_name: String,
    /// The default type argument (e.g. `string`).
    default_type: String,
}

/// Flags type arguments that duplicate the default type parameter.
#[derive(Debug)]
pub struct NoUnnecessaryTypeArguments;

impl LintRule for NoUnnecessaryTypeArguments {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow type arguments that match the default type parameter".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();

        // Phase 1: collect generic types with defaults.
        let generics = collect_generic_defaults(source);
        if generics.is_empty() {
            return;
        }

        // Phase 2: find usage sites passing the default type explicitly.
        let violations = find_redundant_type_args(source, &generics);

        for (span, type_name, default_type) in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "Type argument `{default_type}` is the default for `{type_name}` and can be omitted"
                ),
                span,
                severity: Severity::Warning,
                help: Some(format!("Replace `{type_name}<{default_type}>` with `{type_name}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Remove redundant type argument `<{default_type}>`"),
                    edits: vec![Edit {
                        span,
                        replacement: type_name.clone(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Scan for patterns like `type Foo<T = DefaultType>` or `interface Foo<T = DefaultType>`
/// and extract the type name and its default.
///
/// This is a simplified heuristic that only handles single-parameter generics.
fn collect_generic_defaults(source: &str) -> Vec<GenericDefault> {
    let mut results = Vec::new();

    // Look for `type <Name><` or `interface <Name><` patterns.
    for keyword in &["type ", "interface ", "class "] {
        let mut search_start: usize = 0;
        while let Some(pos) = source.get(search_start..).and_then(|s| s.find(keyword)) {
            let abs_pos = search_start.saturating_add(pos);
            let after_keyword = abs_pos.saturating_add(keyword.len());

            // Extract the type name.
            let remaining = source.get(after_keyword..).unwrap_or("");
            let type_name: String = remaining
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();

            if type_name.is_empty() {
                search_start = after_keyword.saturating_add(1);
                continue;
            }

            let after_name = after_keyword.saturating_add(type_name.len());

            // Check for `<` immediately after the name.
            let next_char = source
                .get(after_name..after_name.saturating_add(1))
                .unwrap_or("");
            if next_char != "<" {
                search_start = after_name.saturating_add(1);
                continue;
            }

            // Find the `= DefaultType>` pattern inside the angle brackets.
            let bracket_content_start = after_name.saturating_add(1);
            let bracket_end = source
                .get(bracket_content_start..)
                .and_then(|s| s.find('>'))
                .map(|p| bracket_content_start.saturating_add(p));

            if let Some(end_pos) = bracket_end {
                let bracket_content = source.get(bracket_content_start..end_pos).unwrap_or("");

                // Look for `= <type>` in the bracket content.
                if let Some(eq_pos) = bracket_content.find('=') {
                    let default_type = bracket_content
                        .get(eq_pos.saturating_add(1)..)
                        .unwrap_or("")
                        .trim()
                        .to_owned();

                    if !default_type.is_empty() {
                        results.push(GenericDefault {
                            type_name: type_name.clone(),
                            default_type,
                        });
                    }
                }
            }

            search_start = after_name.saturating_add(1);
        }
    }

    results
}

/// Find usage sites like `TypeName<DefaultType>` where the type argument
/// matches the default. Returns `(span, type_name, default_type)` tuples.
fn find_redundant_type_args(
    source: &str,
    generics: &[GenericDefault],
) -> Vec<(Span, String, String)> {
    let mut results = Vec::new();

    for generic in generics {
        let pattern = format!("{}<{}>", generic.type_name, generic.default_type);

        let mut search_start: usize = 0;
        while let Some(pos) = source.get(search_start..).and_then(|s| s.find(&pattern)) {
            let abs_pos = search_start.saturating_add(pos);

            // Make sure this is not the definition itself — skip if preceded by
            // `type `, `interface `, or `class `.
            let prefix_start = abs_pos.saturating_sub(12);
            let prefix = source.get(prefix_start..abs_pos).unwrap_or("");
            let is_definition = prefix.ends_with("type ")
                || prefix.ends_with("interface ")
                || prefix.ends_with("class ");

            if !is_definition {
                let start = u32::try_from(abs_pos).unwrap_or(0);
                let end = u32::try_from(abs_pos.saturating_add(pattern.len())).unwrap_or(start);
                results.push((
                    Span::new(start, end),
                    generic.type_name.clone(),
                    generic.default_type.clone(),
                ));
            }

            search_start = abs_pos.saturating_add(1);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    /// Helper to lint TypeScript source code.
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnnecessaryTypeArguments)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_redundant_type_argument() {
        let source = "type Box<T = string> = { value: T };\nlet b: Box<string>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "passing the default type argument should be flagged"
        );
    }

    #[test]
    fn test_allows_different_type_argument() {
        let source = "type Box<T = string> = { value: T };\nlet b: Box<number>;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "passing a non-default type argument should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_type_argument() {
        let source = "type Box<T = string> = { value: T };\nlet b: Box;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "omitting the type argument should not be flagged"
        );
    }

    #[test]
    fn test_flags_interface_default() {
        let source = "interface Container<T = number> { item: T; }\nlet c: Container<number>;";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "passing the default type argument on an interface should be flagged"
        );
    }

    #[test]
    fn test_does_not_flag_definition_itself() {
        let source = "type Box<T = string> = { value: T };";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "the generic definition itself should not be flagged"
        );
    }
}
