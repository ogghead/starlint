//! Rule: `typescript/no-unsafe-enum-comparison`
//!
//! Flags comparisons between enum members and non-enum values using `==` or
//! `===`. Enum values in TypeScript are nominally typed, so comparing them
//! against raw primitives is often a mistake that defeats the purpose of using
//! an enum.
//!
//! Uses a text-based approach: first collects enum declarations and their
//! member names, then scans for comparison operators where one side references
//! a known `EnumName.Member` and the other is a raw literal or non-enum value.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags comparisons of enum members against non-enum values.
#[derive(Debug)]
pub struct NoUnsafeEnumComparison;

impl NativeRule for NoUnsafeEnumComparison {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-enum-comparison".to_owned(),
            description: "Disallow comparing enum members with non-enum values".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();
        let enum_names = collect_enum_names(source);

        if enum_names.is_empty() {
            return;
        }

        // Scan for comparison patterns: `EnumName.Member == <literal>` or
        // `<literal> == EnumName.Member` (including ===, !=, !==).
        let comparisons = find_enum_comparisons(source, &enum_names);

        for (start, end) in comparisons {
            ctx.report(Diagnostic {
                rule_name: "typescript/no-unsafe-enum-comparison".to_owned(),
                message: "Do not compare enum values with non-enum primitives".to_owned(),
                span: Span::new(start, end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Collect the names of all `enum` declarations in the source.
fn collect_enum_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut search_from: usize = 0;

    while let Some(pos) = source.get(search_from..).and_then(|s| s.find("enum ")) {
        let absolute_pos = search_from.saturating_add(pos);
        let after_keyword = absolute_pos.saturating_add(5);

        // Skip if this `enum` keyword is not at a word boundary (e.g. inside
        // an identifier like `xenum`).
        if absolute_pos > 0 {
            let prev_char = source
                .as_bytes()
                .get(absolute_pos.saturating_sub(1))
                .copied();
            if prev_char.is_none_or(|c| c.is_ascii_alphanumeric() || c == b'_') {
                search_from = after_keyword;
                continue;
            }
        }

        // Extract the enum name: skip whitespace, read identifier chars.
        let rest = source.get(after_keyword..).unwrap_or("");
        let trimmed = rest.trim_start();
        let name: String = trimmed
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();

        if !name.is_empty() {
            names.push(name);
        }

        search_from = after_keyword;
    }

    names
}

/// Comparison operators to detect.
const COMPARISON_OPS: &[&str] = &["===", "!==", "==", "!="];

/// Find lines containing comparisons where one side references an enum member
/// (`EnumName.Something`) and the other side is a raw literal.
fn find_enum_comparisons(source: &str, enum_names: &[String]) -> Vec<(u32, u32)> {
    let mut results = Vec::new();
    let mut line_offset: usize = 0;

    for line in source.lines() {
        for op in COMPARISON_OPS {
            if let Some(op_pos) = line.find(op) {
                let raw_lhs = line.get(..op_pos).unwrap_or("");
                let raw_rhs = line.get(op_pos.saturating_add(op.len())..).unwrap_or("");
                let lhs = strip_syntax_noise(raw_lhs);
                let rhs = strip_syntax_noise(raw_rhs);

                let lhs_is_enum = is_enum_member_access(lhs, enum_names);
                let rhs_is_enum = is_enum_member_access(rhs, enum_names);
                let lhs_is_literal = is_primitive_literal(lhs);
                let rhs_is_literal = is_primitive_literal(rhs);

                // Flag when one side is an enum access and the other is a literal.
                if (lhs_is_enum && rhs_is_literal) || (rhs_is_enum && lhs_is_literal) {
                    let start = u32::try_from(line_offset).unwrap_or(0);
                    let end =
                        u32::try_from(line_offset.saturating_add(line.len())).unwrap_or(start);
                    results.push((start, end));
                    break; // one diagnostic per line
                }
            }
        }
        // +1 for the newline character.
        line_offset = line_offset.saturating_add(line.len()).saturating_add(1);
    }

    results
}

/// Strip surrounding syntax noise (parentheses, braces, semicolons, keywords
/// like `if`, `while`, `return`) to isolate the comparison operand.
fn strip_syntax_noise(s: &str) -> &str {
    let trimmed = s.trim();
    // Strip trailing noise: `)`, `}`, `;`, `{`
    let without_trailing = trimmed.trim_end_matches([')', '}', ';', '{', ' ']);
    // Strip leading noise: `(`, keywords like `if`, `while`, `return`, etc.
    let without_leading = without_trailing.trim_start_matches(['(', ' ']);
    // Strip common leading keywords followed by optional whitespace/parens
    let without_keywords = without_leading
        .strip_prefix("if")
        .or_else(|| without_leading.strip_prefix("while"))
        .or_else(|| without_leading.strip_prefix("return"))
        .unwrap_or(without_leading);
    without_keywords.trim_start_matches(['(', ' ']).trim()
}

/// Check if a string looks like `EnumName.Member`.
fn is_enum_member_access(s: &str, enum_names: &[String]) -> bool {
    if let Some(dot_pos) = s.find('.') {
        let prefix = s.get(..dot_pos).unwrap_or("").trim();
        enum_names.iter().any(|n| n == prefix)
    } else {
        false
    }
}

/// Check if a string looks like a primitive literal (number, string, or boolean).
fn is_primitive_literal(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Numeric literal
    if trimmed.as_bytes().first().is_none_or(u8::is_ascii_digit) && trimmed.parse::<f64>().is_ok() {
        return true;
    }
    // String literal
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return true;
    }
    // Boolean literal
    trimmed == "true" || trimmed == "false"
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafeEnumComparison)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_enum_compared_to_number() {
        let source = "enum Color { Red = 0 }\nif (Color.Red == 0) {}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "comparing enum member to a number literal should be flagged"
        );
    }

    #[test]
    fn test_flags_enum_compared_to_string() {
        let source = r#"enum Status { Active = "active" }
if (Status.Active === "active") {}"#;
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "comparing enum member to a string literal should be flagged"
        );
    }

    #[test]
    fn test_flags_literal_on_left_side() {
        let source = "enum Dir { Up = 1 }\nif (1 === Dir.Up) {}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "literal on left side of enum comparison should be flagged"
        );
    }

    #[test]
    fn test_allows_enum_to_enum_comparison() {
        let source = "enum Dir { Up = 1, Down = 2 }\nif (Dir.Up === Dir.Down) {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "enum-to-enum comparison should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_enum_in_file() {
        let source = "if (x === 1) {}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "files without enums should produce no diagnostics"
        );
    }
}
