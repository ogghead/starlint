//! Rule: `no-this-in-exported-function` (oxc)
//!
//! Flag `this` usage in exported functions (not class methods). Exported
//! functions shouldn't rely on `this` binding — it's fragile and error-prone.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Scan source text for exported function declarations that use `this`.
///
/// Returns a list of spans for each offending exported function.
fn find_exported_functions_with_this(source: &str) -> Vec<Span> {
    let len = source.len();
    let mut pos = 0;
    let mut results = Vec::new();

    while pos < len {
        let remaining = source.get(pos..).unwrap_or("");
        let Some(export_offset) = remaining.find("export") else {
            break;
        };
        let abs_export = pos.saturating_add(export_offset);

        // Ensure it's a word boundary
        let before_ok = abs_export == 0
            || source
                .get(..abs_export)
                .and_then(|s| s.chars().next_back())
                .is_some_and(|c| !c.is_alphanumeric() && c != '_');

        let after_export = abs_export.saturating_add(6);
        if !before_ok {
            pos = after_export;
            continue;
        }

        // Skip whitespace after "export"
        let after_remaining = source.get(after_export..).unwrap_or("");
        let trimmed = after_remaining.trim_start();
        let ws_len = after_remaining.len().saturating_sub(trimmed.len());

        // Check for "function" or "default function" or "async function"
        let func_start;
        if trimmed.starts_with("function") {
            func_start = after_export.saturating_add(ws_len);
        } else if trimmed.starts_with("default") {
            let after_default = trimmed.get(7..).unwrap_or("").trim_start();
            if after_default.starts_with("function") {
                let total_skip = after_export
                    .saturating_add(ws_len)
                    .saturating_add(7)
                    .saturating_add(
                        trimmed
                            .get(7..)
                            .unwrap_or("")
                            .len()
                            .saturating_sub(after_default.len()),
                    );
                func_start = total_skip;
            } else {
                pos = after_export;
                continue;
            }
        } else if trimmed.starts_with("async") {
            let after_async = trimmed.get(5..).unwrap_or("").trim_start();
            if after_async.starts_with("function") {
                let total_skip = after_export
                    .saturating_add(ws_len)
                    .saturating_add(5)
                    .saturating_add(
                        trimmed
                            .get(5..)
                            .unwrap_or("")
                            .len()
                            .saturating_sub(after_async.len()),
                    );
                func_start = total_skip;
            } else {
                pos = after_export;
                continue;
            }
        } else {
            pos = after_export;
            continue;
        }

        // Find the function body opening brace
        let func_remaining = source.get(func_start..).unwrap_or("");
        let Some(brace_offset) = func_remaining.find('{') else {
            pos = after_export.saturating_add(1);
            continue;
        };
        let brace_start = func_start.saturating_add(brace_offset);

        // Find matching closing brace
        let mut depth: u32 = 0;
        let mut scan = brace_start;
        let mut body_end = len;
        while scan < len {
            let ch = source.get(scan..scan.saturating_add(1)).unwrap_or("");
            if ch == "{" {
                depth = depth.saturating_add(1);
            } else if ch == "}" {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    body_end = scan.saturating_add(1);
                    break;
                }
            }
            scan = scan.saturating_add(1);
        }

        // Check body for `this`
        if let Some(body_text) = source.get(brace_start..body_end) {
            if source_contains_this(body_text) {
                let span_start = u32::try_from(abs_export).unwrap_or(0);
                let span_end = u32::try_from(body_end).unwrap_or(0);
                results.push(Span::new(span_start, span_end));
            }
        }

        pos = body_end;
    }

    results
}

/// Flags `this` usage inside exported function declarations.
#[derive(Debug)]
pub struct NoThisInExportedFunction;

/// Check whether a slice of source text contains the `this` keyword
/// used as an identifier (not as part of another word).
fn source_contains_this(source: &str) -> bool {
    let mut rest = source;
    while let Some(pos) = rest.find("this") {
        // Check character before
        let before_ok = pos == 0
            || rest
                .get(..pos)
                .and_then(|s| s.chars().next_back())
                .is_some_and(|c| !c.is_alphanumeric() && c != '_' && c != '$');

        // Check character after
        let after_start = pos.saturating_add(4);
        let after_ok = rest.get(after_start..).is_none_or(|s| {
            s.chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '$')
        });

        if before_ok && after_ok {
            return true;
        }

        rest = rest.get(after_start..).unwrap_or("");
    }
    false
}

impl NativeRule for NoThisInExportedFunction {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-this-in-exported-function".to_owned(),
            description: "Disallow `this` in exported functions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let flagged_spans = find_exported_functions_with_this(ctx.source_text());

        for span in flagged_spans {
            ctx.report(Diagnostic {
                rule_name: "no-this-in-exported-function".to_owned(),
                message: "Exported functions should not use `this`".to_owned(),
                span,
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThisInExportedFunction)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_this_in_exported_function() {
        let diags = lint("export function foo() { this.x; }");
        assert_eq!(
            diags.len(),
            1,
            "exported function using this should be flagged"
        );
    }

    #[test]
    fn test_allows_exported_function_without_this() {
        let diags = lint("export function foo() { return 1; }");
        assert!(
            diags.is_empty(),
            "exported function without this should not be flagged"
        );
    }

    #[test]
    fn test_allows_class_method_with_this() {
        let diags = lint("class A { foo() { this.x; } }");
        assert!(
            diags.is_empty(),
            "class method using this should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_exported_function_with_this() {
        let diags = lint("function foo() { this.x; }");
        assert!(
            diags.is_empty(),
            "non-exported function with this should not be flagged"
        );
    }
}
