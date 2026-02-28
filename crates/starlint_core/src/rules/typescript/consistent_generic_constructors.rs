//! Rule: `typescript/consistent-generic-constructors`
//!
//! Prefer specifying generic type arguments on the constructor call rather
//! than on the type annotation. When a variable is declared with a generic
//! type annotation and initialized with `new`, the type arguments should
//! appear on the `new` expression so that the type flows naturally from
//! the value, e.g. `const x = new Foo<string>()` instead of
//! `const x: Foo<string> = new Foo()`.

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags variable declarations where generic type arguments appear on the
/// type annotation rather than on the constructor call.
#[derive(Debug)]
pub struct ConsistentGenericConstructors;

impl NativeRule for ConsistentGenericConstructors {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-generic-constructors".to_owned(),
            description:
                "Prefer generic type arguments on constructor calls rather than type annotations"
                    .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let findings = find_inconsistent_generics(ctx.source_text());

        for (start, end) in findings {
            ctx.report_warning(
                "typescript/consistent-generic-constructors",
                "Generic type arguments should be on the constructor call, not the type annotation — use `new Foo<T>()` instead",
                Span::new(start, end),
            );
        }
    }
}

/// Scan source text for patterns like `const x: Foo<...> = new Foo()` where
/// the type annotation has generic args but the constructor call does not.
///
/// Returns `(start, end)` byte offsets for each finding.
fn find_inconsistent_generics(source: &str) -> Vec<(u32, u32)> {
    let mut results = Vec::new();

    // Match patterns: `(const|let|var) <name>: <Type><...> = new <Type>(`
    // We look for lines that have both a generic type annotation and a plain
    // `new X()` constructor without type args.
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Must start with a variable declaration keyword
        let rest = if let Some(r) = trimmed.strip_prefix("const ") {
            r
        } else if let Some(r) = trimmed.strip_prefix("let ") {
            r
        } else if let Some(r) = trimmed.strip_prefix("var ") {
            r
        } else {
            continue;
        };

        // Must have a colon (type annotation) and `= new`
        let Some(colon_pos) = rest.find(':') else {
            continue;
        };

        let Some(eq_new_pos) = rest.find("= new ") else {
            continue;
        };

        // The type annotation is between the colon and the `=`
        if colon_pos >= eq_new_pos {
            continue;
        }

        let type_annotation = rest
            .get(colon_pos.saturating_add(1)..eq_new_pos)
            .unwrap_or("")
            .trim();

        // Check that the type annotation has generic args (`<...>`)
        if !type_annotation.contains('<') || !type_annotation.contains('>') {
            continue;
        }

        // The constructor part is after `= new `
        let constructor_part = rest
            .get(eq_new_pos.saturating_add(6)..)
            .unwrap_or("")
            .trim();

        // Check that the constructor call does NOT have generic args before `(`
        let Some(paren_pos) = constructor_part.find('(') else {
            continue;
        };

        let before_paren = constructor_part.get(..paren_pos).unwrap_or("");

        // If the constructor already has `<...>` type args, this is fine
        if before_paren.contains('<') {
            continue;
        }

        // Calculate byte offset for this line in the source
        let line_start = source.lines().take(line_idx).fold(0_usize, |acc, l| {
            acc.saturating_add(l.len()).saturating_add(1)
        });
        let line_end = line_start.saturating_add(line.len());

        let start = u32::try_from(line_start).unwrap_or(0);
        let end = u32::try_from(line_end).unwrap_or(start);
        results.push((start, end));
    }

    results
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentGenericConstructors)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_generic_on_annotation() {
        let diags = lint("const x: Map<string, number> = new Map();");
        assert_eq!(
            diags.len(),
            1,
            "generic type on annotation with plain constructor should be flagged"
        );
    }

    #[test]
    fn test_flags_let_generic_on_annotation() {
        let diags = lint("let items: Array<string> = new Array();");
        assert_eq!(
            diags.len(),
            1,
            "generic on type annotation with plain `new` should be flagged"
        );
    }

    #[test]
    fn test_allows_generic_on_constructor() {
        let diags = lint("const x = new Map<string, number>();");
        assert!(
            diags.is_empty(),
            "generic on constructor call should not be flagged"
        );
    }

    #[test]
    fn test_allows_both_generics() {
        let diags = lint("const x: Map<string, number> = new Map<string, number>();");
        assert!(
            diags.is_empty(),
            "generic on both annotation and constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_generics() {
        let diags = lint("const x = new Map();");
        assert!(diags.is_empty(), "no generics at all should not be flagged");
    }
}
