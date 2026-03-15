//! Rule: `typescript/no-unnecessary-parameter-property-assignment`
//!
//! Disallow unnecessary assignment of constructor parameter properties.
//! TypeScript parameter properties (e.g. `constructor(public x: number)`)
//! automatically assign the parameter to `this.x`. An explicit
//! `this.x = x` in the constructor body is therefore redundant and should
//! be removed.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_rule_framework::{LintContext, LintRule};

/// Flags redundant `this.x = x` assignments in constructors that already
/// use parameter properties.
#[derive(Debug)]
pub struct NoUnnecessaryParameterPropertyAssignment;

impl LintRule for NoUnnecessaryParameterPropertyAssignment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-parameter-property-assignment".to_owned(),
            description: "Disallow unnecessary assignment of constructor parameter properties"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let source = ctx.source_text();
        let findings = find_redundant_param_assignments(source);

        // Collect fix data into owned values to satisfy borrow checker
        let fixes: Vec<_> = findings
            .into_iter()
            .map(|(name, start, end)| {
                let after_end = end as usize;
                let mut delete_end = after_end;
                let remaining = source.get(after_end..).unwrap_or("");
                for ch in remaining.chars() {
                    if ch == ';' || ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                        delete_end = delete_end.saturating_add(ch.len_utf8());
                    } else {
                        break;
                    }
                }
                let mut delete_start = start as usize;
                let before = source.get(..delete_start).unwrap_or("");
                for ch in before.chars().rev() {
                    if ch == ' ' || ch == '\t' {
                        delete_start = delete_start.saturating_sub(ch.len_utf8());
                    } else {
                        break;
                    }
                }
                let fix_start = u32::try_from(delete_start).unwrap_or(start);
                let fix_end = u32::try_from(delete_end).unwrap_or(end);
                (name, start, end, fix_start, fix_end)
            })
            .collect();

        for (name, start, end, fix_start, fix_end) in fixes {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unnecessary-parameter-property-assignment".to_owned(),
                message: format!(
                    "Unnecessary assignment `this.{name} = {name}` — parameter property already assigns it"
                ),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: Some(format!("Remove `this.{name} = {name};`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Remove redundant `this.{name} = {name};`"),
                    edits: vec![Edit {
                        span: Span::new(fix_start, fix_end),
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Accessibility modifiers that create parameter properties in TypeScript.
/// Longer prefixes must appear before shorter ones to ensure correct matching.
const MODIFIERS: &[&str] = &[
    "public readonly ",
    "private readonly ",
    "protected readonly ",
    "public ",
    "private ",
    "protected ",
    "readonly ",
];

/// Scan source text for constructors that have parameter properties and
/// redundant `this.x = x` assignments in the body.
///
/// Returns `(param_name, assign_start, assign_end)` for each redundant assignment.
fn find_redundant_param_assignments(source: &str) -> Vec<(String, u32, u32)> {
    let mut results = Vec::new();
    let mut search_from: usize = 0;

    while let Some(ctor_pos) = source
        .get(search_from..)
        .and_then(|s| s.find("constructor"))
    {
        let absolute_ctor = search_from.saturating_add(ctor_pos);
        search_from = absolute_ctor.saturating_add("constructor".len());

        // Find the parameter list opening paren
        let after_ctor = source.get(search_from..).unwrap_or("");
        let Some(paren_offset) = after_ctor.find('(') else {
            continue;
        };
        let paren_start = search_from.saturating_add(paren_offset);

        // Find matching closing paren
        let Some(close_paren) = find_matching_paren(source, paren_start) else {
            continue;
        };

        let params_str = source
            .get(paren_start.saturating_add(1)..close_paren)
            .unwrap_or("");

        // Extract parameter property names
        let param_names = extract_param_property_names(params_str);
        if param_names.is_empty() {
            continue;
        }

        // Find the constructor body (next `{` after closing paren)
        let after_params = source.get(close_paren..).unwrap_or("");
        let Some(body_brace_offset) = after_params.find('{') else {
            continue;
        };
        let body_start = close_paren.saturating_add(body_brace_offset);

        let Some(body_end) = find_matching_brace(source, body_start) else {
            continue;
        };

        let body = source
            .get(body_start.saturating_add(1)..body_end)
            .unwrap_or("");

        // Check for `this.name = name` in the body
        for name in &param_names {
            let pattern = format!("this.{name} = {name}");
            let mut body_search: usize = 0;
            while let Some(pos) = body.get(body_search..).and_then(|s| s.find(&pattern)) {
                let abs_start = body_start
                    .saturating_add(1)
                    .saturating_add(body_search)
                    .saturating_add(pos);
                let abs_end = abs_start.saturating_add(pattern.len());

                let start = u32::try_from(abs_start).unwrap_or(0);
                let end = u32::try_from(abs_end).unwrap_or(start);
                results.push((name.clone(), start, end));

                body_search = body_search
                    .saturating_add(pos)
                    .saturating_add(pattern.len());
            }
        }

        search_from = body_end.saturating_add(1);
    }

    results
}

/// Extract parameter property names from a constructor parameter list string.
///
/// Looks for parameters prefixed with accessibility modifiers like `public`,
/// `private`, `protected`, or `readonly`.
fn extract_param_property_names(params: &str) -> Vec<String> {
    let mut names = Vec::new();

    for param in params.split(',') {
        let trimmed = param.trim();
        for modifier in MODIFIERS {
            if let Some(rest) = trimmed.strip_prefix(modifier) {
                // The parameter name is the next word (before `:` or `=`)
                let name = rest
                    .split(|c: char| c == ':' || c == '=' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    names.push(name.to_owned());
                }
                break;
            }
        }
    }

    names
}

/// Find the position of the matching closing parenthesis for an opening `(`.
fn find_matching_paren(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, ch) in source.get(open_pos..)?.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos.saturating_add(i));
                }
            }
            _ => {}
        }
    }
    None
}

/// Find the position of the matching closing brace for an opening `{`.
fn find_matching_brace(source: &str, open_pos: usize) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, ch) in source.get(open_pos..)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos.saturating_add(i));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    starlint_rule_framework::lint_rule_test!(NoUnnecessaryParameterPropertyAssignment, "test.ts");

    #[test]
    fn test_flags_redundant_public_assignment() {
        let source = "class Foo { constructor(public x: number) { this.x = x; } }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "redundant this.x = x with public parameter property should be flagged"
        );
    }

    #[test]
    fn test_flags_redundant_private_assignment() {
        let source = "class Bar { constructor(private name: string) { this.name = name; } }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "redundant this.name = name with private parameter property should be flagged"
        );
    }

    #[test]
    fn test_flags_redundant_readonly_assignment() {
        let source = "class Baz { constructor(public readonly id: number) { this.id = id; } }";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "redundant assignment with readonly parameter property should be flagged"
        );
    }

    #[test]
    fn test_allows_no_parameter_property() {
        let source = "class Foo { constructor(x: number) { this.x = x; } }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "regular parameter without modifier should not be flagged"
        );
    }

    #[test]
    fn test_allows_different_assignment() {
        let source = "class Foo { constructor(public x: number) { this.y = x; } }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "assignment to a different property should not be flagged"
        );
    }
}
